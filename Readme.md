TCP3H (TCP Proxy with Proxy Protocol Headers)
================================

This is Very Basic TCP proxy written in rust for testing proxy protocol.

### Summary
This proxy will inject a proxy protocol header [tcp proxy protocol](https://www.haproxy.org/download/1.8/doc/proxy-protocol.txt)
before sending the TCP connection to the downstream server.
The purpose of this is to test downstream server compatibility with proxy protocol v2 headers locally.
It will not work with TLS requests or downstreams.

### To start the proxy
`cargo run --bin tcp3h -- --listen 0.0.0.0:8080  --backend <your apps ip:port>`

### Using the proxy
`curl -vv -X GET <your ip>:8080`

By listening on `0.0.0.0` rather than `127.0.0.1` you can make
requests to your laptops ip. Alternative run this shit and your curl commands
in docker containers.

*disclaimer I needed something quick and simple to use for testing, parts of this were written by ChatGPT
