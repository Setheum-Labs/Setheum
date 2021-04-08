pragma solidity ^0.6.0;

import "./StateRentLib.sol";

contract StateRent {
    event TransferredMaintainer(address indexed contract_address, address indexed new_maintainer);
    
    /**
     * @dev Returns the const of NewContractExtraBytes.
     */
    function newContractExtraBytes() public view returns (uint256) {
        return StateRentLib.newContractExtraBytes();
    }

    /**
     * @dev Returns the const of StorageDepositPerByte.
     */
    function storageDepositPerByte() public view returns (uint256) {
        return StateRentLib.storageDepositPerByte();
    }

    /**
     * @dev Returns the maintainer of the contract.
     */
    function maintainerOf(address contract_address)
        public
        view
        returns (address)
    {
        return StateRentLib.maintainerOf(contract_address);
    }

    /**
     * @dev Returns the const of DeveloperDeposit.
     */
    function developerDeposit() public view returns (uint256) {
        return StateRentLib.developerDeposit();
    }

    /**
     * @dev Returns the const of DeploymentFee.
     */
    function deploymentFee() public view returns (uint256) {
        return StateRentLib.deploymentFee();
    }

    /**
     * @dev Transfer the maintainer of the contract.
     * Returns a boolean value indicating whether the operation succeeded.
     */
    function transferMaintainer(
        address contract_address,
        address new_maintainer
    ) public returns (bool) {
        _transferMaintainer(msg.sender, contract_address, new_maintainer);
        return true;
    }

    function _transferMaintainer(
        address sender,
        address contract_address,
        address new_maintainer
    ) internal {
        require(sender != address(0), "StateRent: the sender is the zero address");
        require(contract_address != address(0), "StateRent: the contract_address is the zero address");
        require(new_maintainer != address(0), "StateRent: the new_maintainer is the zero address");

        StateRentLib.transferMaintainer(msg.sender, contract_address, new_maintainer);
        emit TransferredMaintainer(contract_address, new_maintainer);
    }
}

