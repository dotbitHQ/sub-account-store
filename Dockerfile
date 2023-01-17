FROM ubuntu:bionic

WORKDIR /app

COPY ./build/rpc_server .

VOLUME ["/app/data"]
EXPOSE 9130

ENTRYPOINT ["./rpc_server", "-d", "data", "-l", "0.0.0.0:9130"]
