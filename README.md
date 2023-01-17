## Sparse merkle tree rocksdb store implementation
This is a rust implementation of a [sparse merkle tree](https://github.com/nervosnetwork/sparse-merkle-tree) store using rocksdb as the backend.

## Usage
### Commands
You can specify two parameters:
* `-l` specifies the listening address and port, the default is `127.0.0.1:10000`
* `-s` specifies the path to the store database, the default is `/tmp/smt-store`
### Docker
Depending on your installation environment, you may need to add `sudo` to obtain authorization.
```shell
make docker-build
make docker-image
make docker-test
```

## Examples

### Commands
```
cargo run -l 127.0.0.1:10000 -s /tmp/smt-store-path
```
### RPC request
Here are some sample rpc requests for reference.

#### update_memory_smt

```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "update_memory_smt",
    "params": {
        "opt":{
            "get_proof":true,
            "get_root":true
        },
        "smt_name":"",
        "data":[
            {
                "key":  "0000000000000000000000000000000000000000000000000000000000000000",
                "value":"2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a"
            },
            {
                "key":"2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b",
                "value":"2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"
            },
            {
                "key":"2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d",
                "value":"2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d"
            },
            {
                "key":"1111111111111111111111111111111111111111111111111111111111111111",
                "value":"3333333333333333333333333333333333333333333333333333333333333333"
            },
            {
                "key":"3333333333333333333333333333333333333333333333333333333333333333",
                "value":"1111111111111111111111111111111111111111111111111111111111111111"
            },
            {
                "key":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                "value":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            }
        ]
    }
}' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:10000
```

#### update_db_smt

```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "update_db_smt",
    "params": {
        "opt":{
            "get_proof":true,
            "get_root":true
        },
        "smt_name":"tree101",
        "data":[
            {
                "key":  "0000000000000000000000000000000000000000000000000000000000000000",
                "value":"00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            }
        ]
    }
}' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:10000
```

#### update_db_smt_middle
```shell
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "update_db_smt_middle",
    "params": {
        "opt":{
            "get_proof":true,
            "get_root":true
        },
        "smt_name":"tree101",
        "data":[
            {
                "key":  "0000000000000000000000000000000000000000000000000000000000000000",
                "value":"00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            },
            {
                "key":  "0100000000000000000000000000000000000000000000000000000000000000",
                "value":"11ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            },
            {
                "key":  "0300000000000000000000000000000000000000000000000000000000000000",
                "value":"33ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            },
            {
                "key":  "0400000000000000000000000000000000000000000000000000000000000000",
                "value":"44ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            }
        ]
    }
}' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:10000
```
#### get_smt_root

```shell
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_smt_root",
    "params": {
        "smt_name":"tree101"
    }
}' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:10000
```

#### delete_smt

```shell
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "delete_smt",
    "params": {
        "smt_name":"tree100"
    }
}' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:10000
```

