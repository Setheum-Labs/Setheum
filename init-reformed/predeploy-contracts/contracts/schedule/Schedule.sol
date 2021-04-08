pragma solidity ^0.6.0;

import "./ISchedule.sol";
import "./ScheduleLib.sol";

contract Schedule is ISchedule {
    /**
     * @dev Schedule call the contract.
     * Returns a boolean value indicating whether the operation succeeded.
     */
    function scheduleCall(
        address contract_address,
        uint256 value,
        uint256 gas_limit,
        uint256 storage_limit,
        uint256 min_delay,
        bytes memory input_data
    ) public override returns (bool) {
        _scheduleCall(msg.sender, contract_address, value, gas_limit, storage_limit, min_delay, input_data);
        return true;
    }

    /**
     * @dev Cancel schedule call the contract.
     * Returns a boolean value indicating whether the operation succeeded.
     */
    function cancelCall(
        bytes memory task_id
    ) public override returns (bool) {
        _cancelCall(msg.sender, task_id);
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
        _rescheduleCall(msg.sender, min_delay, task_id);
        return true;
    }

    function _scheduleCall(
        address sender,
        address contract_address,
        uint256 value,
        uint256 gas_limit,
        uint256 storage_limit,
        uint256 min_delay,
        bytes memory input_data
    ) internal {
        require(sender != address(0), "ScheduleCall: the sender is the zero address");
        require(contract_address != address(0), "ScheduleCall: the contract_address is the zero address");
        require(input_data.length > 0, "ScheduleCall: input is null");

        bytes memory task_id = ScheduleCallLib.scheduleCall(msg.sender, contract_address, value, gas_limit, storage_limit, min_delay, input_data);
        emit ScheduledCall(msg.sender, contract_address, task_id);
    }

    function _cancelCall(
        address sender,
        bytes memory task_id
    ) internal {
        require(sender != address(0), "ScheduleCall: the sender is the zero address");

        ScheduleCallLib.cancelCall(msg.sender, task_id);
        emit CanceledCall(msg.sender, task_id);
    }

    function _rescheduleCall(
        address sender,
        uint256 min_delay,
        bytes memory task_id
    ) internal {
        require(sender != address(0), "ScheduleCall: the sender is the zero address");

        ScheduleCallLib.rescheduleCall(msg.sender, min_delay, task_id);
        emit RescheduledCall(msg.sender, task_id);
    }
}

