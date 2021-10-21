use ethereum_abi::Abi;
use lazy_static::lazy_static;
use serde_json::json;

lazy_static! {
    pub static ref ELLIPTICOIN_ABI: Abi = serde_json::from_str(
        &json!(
        [
            {
                "inputs": [
                    {
                        "internalType": "int64",
                        "name": "",
                        "type": "int64"
                    },
                    {
                        "internalType": "address",
                        "name": "",
                        "type": "address"
                    }
                ],
                "name": "addLiquidity",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {
                        "internalType": "int64",
                        "name": "",
                        "type": "int64"
                    },
                    {
                        "internalType": "address",
                        "name": "",
                        "type": "address"
                    },
                    {
                        "internalType": "int64",
                        "name": "",
                        "type": "int64"
                    }
                ],
                "name": "buy",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {
                        "internalType": "int64",
                        "name": "",
                        "type": "int64"
                    },
                    {
                        "internalType": "address",
                        "name": "",
                        "type": "address"
                    },
                    {
                        "internalType": "int64",
                        "name": "",
                        "type": "int64"
                    }
                ],
                "name": "createPool",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {
                        "internalType": "int64",
                        "name": "",
                        "type": "int64"
                    },
                    {
                        "internalType": "address",
                        "name": "",
                        "type": "address"
                    }
                ],
                "name": "createWithdrawlRequest",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {
                        "components": [
                            {
                                "internalType": "enum Playground.PolygonMessageType",
                                "name": "_type",
                                "type": "uint8"
                            },
                            {
                                "internalType": "int64",
                                "name": "amount",
                                "type": "int64"
                            },
                            {
                                "internalType": "address",
                                "name": "token",
                                "type": "address"
                            },
                            {
                                "internalType": "address",
                                "name": "_address",
                                "type": "address"
                            },
                            {
                                "internalType": "int64",
                                "name": "withdrawlId",
                                "type": "int64"
                            },
                            {
                                "internalType": "bytes32",
                                "name": "transactionHash",
                                "type": "bytes32"
                            }
                        ],
                        "internalType": "struct Playground.PolygonMessage[]",
                        "name": "messages",
                        "type": "tuple[]"
                    }
                ],
                "name": "processPolygonMessages",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {
                        "internalType": "int64",
                        "name": "",
                        "type": "int64"
                    },
                    {
                        "internalType": "address",
                        "name": "",
                        "type": "address"
                    }
                ],
                "name": "removeLiquidity",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {
                        "internalType": "bytes32",
                        "name": "",
                        "type": "bytes32"
                    }
                ],
                "name": "seal",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {
                        "internalType": "int64",
                        "name": "",
                        "type": "int64"
                    },
                    {
                        "internalType": "address",
                        "name": "",
                        "type": "address"
                    },
                    {
                        "internalType": "int64",
                        "name": "",
                        "type": "int64"
                    }
                ],
                "name": "sell",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {
                        "internalType": "string",
                        "name": "",
                        "type": "string"
                    },
                    {
                        "internalType": "bytes32",
                        "name": "",
                        "type": "bytes32"
                    },
                    {
                        "internalType": "int64",
                        "name": "",
                        "type": "int64"
                    }
                ],
                "name": "startMining",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
        {
                "constant": false,
                "inputs": [
                    {
                        "name": "_to",
                        "type": "address"
                    },
                    {
                        "name": "_value",
                        "type": "uint256"
                    }
                ],
                "name": "transfer",
                "outputs": [
                    {
                        "name": "",
                        "type": "bool"
                    }
                ],
                "payable": false,
                "stateMutability": "nonpayable",
                "type": "function"
            },
        ]
                                )
        .to_string()
    )
    .unwrap();
}
