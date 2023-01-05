## Sparse merkle tree rocksdb store implementation
This is a rust implementation of a [sparse merkle tree](https://github.com/nervosnetwork/sparse-merkle-tree) store using rocksdb as the backend.

### Usage
Please refer to the unit tests for usage examples.

### Examples

#### Start a rocksdb store backed sparse merkle tree

```
cargo run --example rpc_server -- /tmp/smt-store-dir 127.0.0.1:10000
```

call rpc server to update the tree
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "update_all",
    "params": [
        [
            ["2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a", "2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a"],
            ["2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b", "2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"],
            ["2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"],
            ["1111111111111111111111111111111111111111111111111111111111111111", "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"],
            ["3333333333333333333333333333333333333333333333333333333333333333", "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"]
        ]
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```

call rpc server to get the proof:
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "merkle_proof",
    "params": [
        ["2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b", "1111111111111111111111111111111111111111111111111111111111111111"]
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```

#### Start a rocksdb store backed with multiple sparse merkle trees

```
cargo run --example rpc_server_multi_tree -- /tmp/smt-store-dir 127.0.0.1:10000
```

call rpc server to update the tree
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "update_all",
    "params": [
        "tree1",
        [
            ["2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a", "2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a"],
            ["2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b", "2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"],
            ["2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"],
            ["1111111111111111111111111111111111111111111111111111111111111111", "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"],
            ["3333333333333333333333333333333333333333333333333333333333333333", "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"]
        ]
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```

call rpc server to get the proof:
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "merkle_proof",
    "params": [
        "tree1",
        ["2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b", "1111111111111111111111111111111111111111111111111111111111111111"]
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```


call rpc server to update the tree
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "buildMemorySmt",
    "params": [
        true,
        true,
        [
            ["2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a", "2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a"],
            ["2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b", "2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"],
            ["2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"],
            ["1111111111111111111111111111111111111111111111111111111111111111", "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"],
            ["3333333333333333333333333333333333333333333333333333333333333333", "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"]
        ]
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```

update one key
call rpc server to update the tree
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "updateRocksDbSmt",
    "params": [
        "2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a",
        "3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a",
        "tree1",
        true,
        true
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```


get smt root
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "RocksdbSmtRoot",
    "params": [
        "smt_name" : "tree1"
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```


get smt root
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "RocksdbSmtRoot2",
    "params": {
        "smt_name" : "tree1",
        "smt_name2" : "tree2",
        "data":[
        {
            "key":"2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a",
            "value":"2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a"
        },
        {
            "key":"3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a",
            "value":"3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a"
        }
        ]
    }
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```


!!!!!!!!!!!!!!!!!!
build memory smt 
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
http://localhost:10000
```

 get root 
```
echo '{
"id": 2,
"jsonrpc": "2.0",
"method": "get_smt_root",
"params": {
"smt_name" : "tree1"
}
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```



go test case 
```shell
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "update_all",
    "params": [
      "tree100",
        [
            ["0000000000000000000000000000000000000000000000000000000000000000", "00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"],
            ["0100000000000000000000000000000000000000000000000000000000000000", "11ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"],
            ["0200000000000000000000000000000000000000000000000000000000000000", "22ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"],
            ["0300000000000000000000000000000000000000000000000000000000000000", "33ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"]
        ]
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```

go test case2 
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
                "value":"00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            },
            {
                "key":"0100000000000000000000000000000000000000000000000000000000000000",
                "value":"11ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            },
            {
                "key":"0200000000000000000000000000000000000000000000000000000000000000",
                "value":"22ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            },
            {
                "key":"0300000000000000000000000000000000000000000000000000000000000000",
                "value":"33ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            }
        ]
    }
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```
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
                "value":"00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            },
            {
                "key":"0100000000000000000000000000000000000000000000000000000000000000",
                "value":"11ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            },
            {
                "key":"0200000000000000000000000000000000000000000000000000000000000000",
                "value":"22ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            },
            {
                "key":"0300000000000000000000000000000000000000000000000000000000000000",
                "value":"33ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            }
        ]
    }
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```




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
http://localhost:10000
```


```echo '{
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
                "value":"00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            }
        ]
    }
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```


go test proof 
```shell
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
http://localhost:10000
```

remove tree
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
http://localhost:10000
```


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
http://localhost:10000
```