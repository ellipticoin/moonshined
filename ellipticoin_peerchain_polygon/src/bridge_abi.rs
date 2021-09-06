use ethereum_abi::Abi;
use lazy_static::lazy_static;
use serde_json::json;

lazy_static! {
    pub static ref BRIDGE_ABI: Abi = serde_json::from_str(
        &json!(
        [
          {
            "inputs": [
              {
                "internalType": "address",
                "name": "admin",
                "type": "address"
              },
              {
                "internalType": "contract WMATIC",
                "name": "__WMATIC",
                "type": "address"
              }
            ],
            "stateMutability": "nonpayable",
            "type": "constructor"
          },
          {
            "anonymous": false,
            "inputs": [
              {
                "indexed": true,
                "internalType": "bytes32",
                "name": "role",
                "type": "bytes32"
              },
              {
                "indexed": true,
                "internalType": "bytes32",
                "name": "previousAdminRole",
                "type": "bytes32"
              },
              {
                "indexed": true,
                "internalType": "bytes32",
                "name": "newAdminRole",
                "type": "bytes32"
              }
            ],
            "name": "RoleAdminChanged",
            "type": "event"
          },
          {
            "anonymous": false,
            "inputs": [
              {
                "indexed": true,
                "internalType": "bytes32",
                "name": "role",
                "type": "bytes32"
              },
              {
                "indexed": true,
                "internalType": "address",
                "name": "account",
                "type": "address"
              },
              {
                "indexed": true,
                "internalType": "address",
                "name": "sender",
                "type": "address"
              }
            ],
            "name": "RoleGranted",
            "type": "event"
          },
          {
            "anonymous": false,
            "inputs": [
              {
                "indexed": true,
                "internalType": "bytes32",
                "name": "role",
                "type": "bytes32"
              },
              {
                "indexed": true,
                "internalType": "address",
                "name": "account",
                "type": "address"
              },
              {
                "indexed": true,
                "internalType": "address",
                "name": "sender",
                "type": "address"
              }
            ],
            "name": "RoleRevoked",
            "type": "event"
          },
          {
            "anonymous": false,
            "inputs": [
              {
                "indexed": false,
                "internalType": "int64",
                "name": "withdrawId",
                "type": "int64"
              },
              {
                "indexed": true,
                "internalType": "address",
                "name": "to",
                "type": "address"
              },
              {
                "indexed": false,
                "internalType": "contract ERC20",
                "name": "token",
                "type": "address"
              },
              {
                "indexed": false,
                "internalType": "int64",
                "name": "amount",
                "type": "int64"
              }
            ],
            "name": "Withdrawal",
            "type": "event"
          },
          {
            "inputs": [],
            "name": "DEFAULT_ADMIN_ROLE",
            "outputs": [
              {
                "internalType": "bytes32",
                "name": "",
                "type": "bytes32"
              }
            ],
            "stateMutability": "view",
            "type": "function"
          },
          {
            "inputs": [],
            "name": "VALIDATOR_ROLE",
            "outputs": [
              {
                "internalType": "bytes32",
                "name": "",
                "type": "bytes32"
              }
            ],
            "stateMutability": "view",
            "type": "function"
          },
          {
            "inputs": [],
            "name": "_WMATIC",
            "outputs": [
              {
                "internalType": "contract WMATIC",
                "name": "",
                "type": "address"
              }
            ],
            "stateMutability": "view",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "bytes32",
                "name": "role",
                "type": "bytes32"
              }
            ],
            "name": "getRoleAdmin",
            "outputs": [
              {
                "internalType": "bytes32",
                "name": "",
                "type": "bytes32"
              }
            ],
            "stateMutability": "view",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "bytes32",
                "name": "role",
                "type": "bytes32"
              },
              {
                "internalType": "address",
                "name": "account",
                "type": "address"
              }
            ],
            "name": "grantRole",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "bytes32",
                "name": "role",
                "type": "bytes32"
              },
              {
                "internalType": "address",
                "name": "account",
                "type": "address"
              }
            ],
            "name": "hasRole",
            "outputs": [
              {
                "internalType": "bool",
                "name": "",
                "type": "bool"
              }
            ],
            "stateMutability": "view",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "contract ERC20",
                "name": "token",
                "type": "address"
              },
              {
                "internalType": "address payable",
                "name": "to",
                "type": "address"
              },
              {
                "internalType": "int64",
                "name": "amount",
                "type": "int64"
              },
              {
                "internalType": "int64",
                "name": "withdrawId",
                "type": "int64"
              }
            ],
            "name": "processWithdraw",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "bytes32",
                "name": "role",
                "type": "bytes32"
              },
              {
                "internalType": "address",
                "name": "account",
                "type": "address"
              }
            ],
            "name": "renounceRole",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "bytes32",
                "name": "role",
                "type": "bytes32"
              },
              {
                "internalType": "address",
                "name": "account",
                "type": "address"
              }
            ],
            "name": "revokeRole",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "int64",
                "name": "amount",
                "type": "int64"
              },
              {
                "internalType": "contract ERC20",
                "name": "token",
                "type": "address"
              }
            ],
            "name": "scaleUp",
            "outputs": [
              {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
              }
            ],
            "stateMutability": "view",
            "type": "function"
          },
          {
            "inputs": [],
            "name": "selfDestruct",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "bytes4",
                "name": "interfaceId",
                "type": "bytes4"
              }
            ],
            "name": "supportsInterface",
            "outputs": [
              {
                "internalType": "bool",
                "name": "",
                "type": "bool"
              }
            ],
            "stateMutability": "view",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "address",
                "name": "to",
                "type": "address"
              },
              {
                "internalType": "contract ERC20",
                "name": "token",
                "type": "address"
              },
              {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
              }
            ],
            "name": "withdraw",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
          },
          {
            "stateMutability": "payable",
            "type": "receive"
          }
        ]

                                )
        .to_string()
    )
    .unwrap();
}
