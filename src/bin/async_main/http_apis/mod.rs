use embassy_net::{
    Stack,
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};
use reqwless::client::HttpClient;

#[cfg(feature = "isdayoff")]
pub mod isdayoff;
#[cfg(feature = "weather")]
pub mod weather;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.init_with(|| $val);
        x
    }};
}

pub type HttpClientConcrete =
    HttpClient<'static, TcpClient<'static, 1, 4096, 4096>, DnsSocket<'static>>;

#[cfg(feature = "isdayoff")]
pub fn init_tcp_http(net_stack: Stack<'static>) -> &'static mut HttpClientConcrete {
    info!("TCP/HTTP Client init");
    let tcp_state =
        mk_static!(TcpClientState<1, 4096, 4096>, {TcpClientState::<1, 4096, 4096>::new()});
    let tcp_client = mk_static!(TcpClient<1, 4096, 4096>, {
        TcpClient::new(net_stack, &*tcp_state)
    });
    let dns_socket = mk_static!(DnsSocket<'static>, DnsSocket::new(net_stack));

    mk_static!(HttpClientConcrete, {
        reqwless::client::HttpClient::new(&*tcp_client, &*dns_socket)
    })
}
