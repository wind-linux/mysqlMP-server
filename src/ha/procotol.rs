/*
@author: xiao cai niao
@datetime: 2019/11/15
*/
use serde::Serialize;
use serde::Deserialize;
use std::net::TcpStream;
use std::error::Error;
use std::io::{Read, Write};

#[derive(Debug, Serialize)]
pub enum  MyProtocol {
    MysqlCheck,
    GetMonitor,
    GetSlowLog,
    GetAuditLog,
    SetMaster,          //设置本机为新master
    ChangeMaster,
    SyncBinlog,         //mysql服务宕机，同步差异binlog到新master
    RecoveryCluster,    //宕机重启自动恢复主从同步, 如有差异将回滚本机数据，并保存回滚数据
    RecoveryValue,      //宕机恢复回滚的数据，回给服务端保存，有管理员人工决定是否追加
    ReplicationStatus,  //获取slave同步状态
    DownNodeCheck,      //宕机节点状态检查，用于server端检测到宕机时，分发到各client复检
    Ok,
    Error,
    UnKnow
}
impl MyProtocol {
    pub fn new(code: &u8) -> MyProtocol{
        if code == &0xfe {
            return MyProtocol::MysqlCheck;
        }else if code == &0xfd {
            return MyProtocol::GetMonitor;
        }else if code == &0xfc {
            return MyProtocol::GetSlowLog;
        }else if code == &0xfb {
            return MyProtocol::GetAuditLog;
        }else if code == &0xfa {
            return MyProtocol::SetMaster;
        }else if code == &0xf9 {
            return MyProtocol::ChangeMaster;
        }else if code == &0xf8 {
            return MyProtocol::SyncBinlog;
        }else if code == &0xf7 {
            return MyProtocol::RecoveryCluster;
        }else if code == &0x00 {
            return MyProtocol::Ok;
        }else if code == &0x09 {
            return MyProtocol::Error;
        }else if code == &0xf6 {
            return MyProtocol::RecoveryValue;
        }else if code == &0xf5 {
            return MyProtocol::ReplicationStatus;
        }else if code == &0xf4 {
            return MyProtocol::DownNodeCheck;
        }
        else {
            return MyProtocol::UnKnow;
        }
    }

    pub fn get_code(&self) -> u8 {
        match self {
            MyProtocol::MysqlCheck => 0xfe,
            MyProtocol::GetMonitor => 0xfd,
            MyProtocol::GetSlowLog => 0xfc,
            MyProtocol::GetAuditLog => 0xfb,
            MyProtocol::SetMaster => 0xfa,
            MyProtocol::ChangeMaster => 0xf9,
            MyProtocol::SyncBinlog => 0xf8,
            MyProtocol::RecoveryCluster => 0xf7,
            MyProtocol::Ok => 0x00,
            MyProtocol::Error => 0x09,
            MyProtocol::RecoveryValue => 0xf6,
            MyProtocol::ReplicationStatus => 0xf5,
            MyProtocol::DownNodeCheck => 0xf4,
            MyProtocol::UnKnow => 0xff
        }
    }

}
///
/// 错误信息
///
#[derive(Serialize, Deserialize)]
pub struct ReponseErr{
    pub err: String
}
///
/// client回复服务端状态检查的包
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MysqlState {
    pub online: bool,
    pub role: String,
    pub sql_thread: bool,
    pub io_thread: bool,
    pub seconds_behind: usize,
    pub error: String
}

///
/// 分发到client请求检查宕机节点状态
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownNodeCheck {
    pub host: String,
    pub dbport: usize,
}
impl DownNodeCheck {
    pub fn new(host: String, dbport: usize) -> DownNodeCheck {
        DownNodeCheck{
            host,
            dbport
        }
    }
}

///
/// client回复检查宕机节点状态数据
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownNodeCheckStatus {
    pub host: String,
    pub client_status: bool,
    pub db_status: bool
}

impl DownNodeCheckStatus {
    pub fn new() -> DownNodeCheckStatus {
        DownNodeCheckStatus{
            host: "".to_string(),
            client_status: false,
            db_status: false
        }
    }
}

///
/// 用于master宕机对所有slave请求replication的情况，供选举以及追加binlog的作用
///
#[derive(Serialize, Deserialize, Debug)]
pub struct ReplicationState {
    pub log_name: String,
    pub read_log_pos: usize,
}

///
/// master宕机后新master开始的binlog信息， 供恢复时建立同步使用
#[derive(Serialize, Deserialize, Debug)]
pub struct SetMasterStatus {
    pub log_name: String,
    pub log_pos: String,
    pub gtid: String
}

///
/// 服务端通过该协议请求client查询差异biogln
///
#[derive(Deserialize, Serialize)]
pub struct SyncBinlogInfo{
    binlog: String,
    position: usize
}
///
/// mysql实例宕机client发送所有差异的binlog原始数据
///
#[derive(Serialize, Deserialize)]
pub struct BinlogValue{
    value: Vec<u8>
}
///
/// 主从切换，指向到新master的基础信息
///
#[derive(Deserialize, Serialize)]
pub struct ChangeMasterInfo{
    pub master_host: String,
    pub master_port: String
}

///
/// 宕机恢复服务端发送恢复主从的基础信息
///
#[derive(Deserialize, Debug, Serialize)]
pub struct RecoveryInfo {
    pub binlog: String,
    pub position: usize,
    pub gtid: String,
    pub masterhost: String,
    pub masterport: usize,
}

///
///用于宕机恢复旧master回滚，该结构体是从client发回的回滚等数据信息
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TractionValue{
    //pub event: Vec<Traction>,
    pub cur_sql: Vec<String>,
    pub rollback_sql: Vec<String>,
}
///
///用于宕机恢复旧master回滚，该结构体是从client发回的回滚等数据信息
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowsSql {
    pub sqls: Vec<TractionValue>,
    pub error: String
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostInfoValue {
    pub host: String,   //127.0.0.1:3306
    pub dbport: usize,  //default 3306
    pub rtype: String,  //db、route
    pub cluster_name: String,   //集群名称,route类型默认default
    pub online: bool,   //db是否在线， true、false
    pub insert_time: i64,
    pub update_time: i64,
    pub maintain: bool, //是否处于维护模式，true、false
}

///
/// 用于web拉取节点信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostInfoValueGetAllState {
    pub host: String,   //127.0.0.1:3306
    pub dbport: usize,  //default 3306
    pub rtype: String,  //db、route
    pub cluster_name: String,   //集群名称,route类型默认default
    pub online: bool,   //是否在线， true、false
    pub maintain: bool, //是否处于维护模式，true、false
    pub role: String,   //主从角色
}

impl HostInfoValueGetAllState {
    pub fn new(host_info: &HostInfoValue,role: String) -> HostInfoValueGetAllState {
        HostInfoValueGetAllState{
            host: host_info.host.clone(),
            dbport: host_info.dbport.clone(),
            rtype: host_info.rtype.clone(),
            cluster_name: host_info.cluster_name.clone(),
            online: host_info.online.clone(),
            maintain: host_info.maintain.clone(),
            role
        }
    }
}



fn header(code: u8, payload: u64) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![];
    buf.push(code);
    let payload = crate::readvalue::write_u64(payload);
    buf.extend(payload);
    return buf;
}

pub fn send_packet(packet: &Vec<u8>, conn: &mut TcpStream) -> Result<(), Box<dyn Error>>{
    conn.write(packet)?;
    conn.flush()?;
    Ok(())
}

pub fn send_value_packet<T: Serialize>(mut tcp: &TcpStream, value: &T, type_code: MyProtocol) -> Result<(), Box<dyn Error>> {
    let value = serde_json::to_string(value)?;
    let mut buf = header(type_code.get_code(), value.len() as u64);
    buf.extend(value.as_bytes());
    tcp.write(buf.as_ref())?;
    tcp.flush()?;
    Ok(())
}

///
/// 接收client返回的数据
///
pub fn rec_packet(conn: &mut TcpStream) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf: Vec<u8> = vec![];
    let mut header: Vec<u8> = vec![0u8;9];
    conn.read_exact(&mut header)?;
    let payload = crate::readvalue::read_u64(&header[1..]);
    let mut payload_buf: Vec<u8> = vec![0u8; payload as usize];
    conn.read_exact(&mut payload_buf)?;
    buf.extend(header);
    buf.extend(payload_buf);
    Ok(buf)
}