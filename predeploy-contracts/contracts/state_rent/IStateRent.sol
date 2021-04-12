pragma solidity ^0.6.0;

interface IStateRent {
    event TransferredMaintainer(address indexed contract_address, address indexed new_maintainer);

    // Returns the const of NewContractExtraBytes.
    function newContractExtraBytes() external returns (uint256);

    // Returns the const of StorageDepositPerByte.
    function storageDepositPerByte() external returns (uint256);

    // Returns the maintainer of the contract.
    function maintainerOf(address contract_address) external returns (address);

    // Returns the const of DeveloperDeposit.
    function developerDeposit() external returns (uint256);

    // Returns the const of DeploymentFee.
    function deploymentFee() external returns (uint256);

    // Transfer the maintainer of the contract.
    // Returns a boolean value indicating whether the operation succeeded.
    function transferMaintainer(address contract_address, address new_maintainer) external returns (bool);
}
