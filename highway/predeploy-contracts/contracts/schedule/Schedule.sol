// SPDX-License-Identifier: GPL-3.0-or-later

pragma solidity ^0.8.0;

import "./ISchedule.sol";

contract Schedule is ISchedule {
    address constant private precompile = address(0x0000000000000000000000000000000000000404);

    /**
     * @dev Schedule call the contract.
     * Returns a bytes value equal to the task_id of the task created.
     */
    function scheduleCall(
        address contract_address,
        uint256 value,
        uint256 gas_limit,
        uint256 storage_limit,
        uint256 min_delay,
        bytes memory input_data
    ) public override returns (bytes memory) {
        require(contract_address != address(0), "ScheduleCall: the contract_address is the zero address");
        require(input_data.length > 0, "ScheduleCall: input is null");

        (bool success, bytes memory returnData) = precompile.call(abi.encodeWithSignature("scheduleCall(address,address,uint256,uint256,uint256,bytes)", msg.sender, contract_address, value, gas_limit, storage_limit, min_delay, input_data));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        (bytes memory task_id) = abi.decode(returnData, (bytes));

        emit ScheduledCall(msg.sender, contract_address, task_id);
        return task_id;
    }

    /**
     * @dev Cancel schedule call the contract.
     * Returns a boolean value indicating whether the operation succeeded.
     */
    function cancelCall(
        bytes memory task_id
    ) public override returns (bool) {
        (bool success, bytes memory returnData) = precompile.call(abi.encodeWithSignature("cancelCall(address,bytes)", msg.sender, task_id));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        emit CanceledCall(msg.sender, task_id);
        return true;
    }

    /**
     * @dev Reschedule call the contract.
     * Returns a boolean value indicating whether the operation succeeded.
     */
    function rescheduleCall(
        uint256 min_delay,
        bytes memory task_id
    ) public override returns (bool) {
        (bool success, bytes memory returnData) = precompile.call(abi.encodeWithSignature("rescheduleCall(address,uint256,bytes)", msg.sender, min_delay, task_id));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        emit RescheduledCall(msg.sender, task_id);
        return true;
    }
}
