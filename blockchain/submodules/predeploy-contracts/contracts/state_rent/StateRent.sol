// SPDX-License-Identifier: GPL-3.0-or-later

pragma solidity ^0.8.0;

import "./IStateRent.sol";

contract StateRent is IStateRent {
    address constant private precompile = address(0x0000000000000000000000000000000000000402);

    /**
     * @dev Returns the const of NewContractExtraBytes.
     */
    function newContractExtraBytes() public view override returns (uint256) {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodeWithSignature("newContractExtraBytes()"));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (uint256));
    }

    /**
     * @dev Returns the const of StorageDepositPerByte.
     */
    function storageDepositPerByte() public view override returns (uint256) {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodeWithSignature("storageDepositPerByte()"));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (uint256));
    }

    /**
     * @dev Returns the maintainer of the contract.
     */
    function maintainerOf(address contract_address)
        public
        view
        override
        returns (address)
    {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodeWithSignature("maintainerOf(address)", contract_address));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (address));
    }

    /**
     * @dev Returns the const of DeveloperDeposit.
     */
    function developerDeposit() public view override returns (uint256) {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodeWithSignature("developerDeposit()"));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (uint256));
    }

    /**
     * @dev Returns the const of DeploymentFee.
     */
    function deploymentFee() public view override returns (uint256) {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodeWithSignature("deploymentFee()"));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (uint256));
    }

    /**
     * @dev Transfer the maintainer of the contract.
     * Returns a boolean value indicating whether the operation succeeded.
     */
    function transferMaintainer(
        address contract_address,
        address new_maintainer
    ) public override returns (bool) {
        require(contract_address != address(0), "StateRent: the contract_address is the zero address");
        require(new_maintainer != address(0), "StateRent: the new_maintainer is the zero address");

        (bool success, bytes memory returnData) = precompile.call(abi.encodeWithSignature("transferMaintainer(address,address,address)", msg.sender, contract_address, new_maintainer));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        emit TransferredMaintainer(contract_address, new_maintainer);
        return true;
    }
}
