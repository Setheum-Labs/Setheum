// SPDX-License-Identifier: GPL-3.0-or-later

pragma solidity ^0.8.0;

interface IStateRent {
    event TransferredMaintainer(address indexed contract_address, address indexed new_maintainer);

    // Returns the const of NewContractExtraBytes.
    function newContractExtraBytes() external view returns (uint256);

    // Returns the const of StorageDepositPerByte.
    function storageDepositPerByte() external view returns (uint256);

    // Returns the maintainer of the contract.
    function maintainerOf(address contract_address) external view returns (address);

    // Returns the const of DeveloperDeposit.
    function developerDeposit() external view returns (uint256);

    // Returns the const of DeploymentFee.
    function deploymentFee() external view returns (uint256);

    // Transfer the maintainer of the contract.
    // Returns a boolean value indicating whether the operation succeeded.
    function transferMaintainer(address contract_address, address new_maintainer) external returns (bool);
}
