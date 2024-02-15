// SPDX-License-Identifier: GPL-3.0-or-later

pragma solidity ^0.8.0;

library MultiCurrency {
    address constant private precompile = address(0x0000000000000000000000000000000000000400);

    function name() internal view returns (string memory) {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodeWithSignature("name()"));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (string));
    }

    function symbol() internal view returns (string memory) {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodeWithSignature("symbol()"));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (string));
    }

    function decimals() internal view returns (uint8) {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodeWithSignature("decimals()"));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (uint8));
    }

    function totalSupply() internal view returns (uint256) {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodeWithSignature("totalSupply()"));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (uint256));
    }

    function balanceOf(address account) internal view returns (uint256) {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodeWithSignature("balanceOf(address)", account));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (uint256));
    }

    function transfer(address sender, address recipient, uint256 amount) internal {
        (bool success, bytes memory returnData) = precompile.call(abi.encodeWithSignature("transfer(address,address,uint256)", sender, recipient, amount));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }
    }
}
