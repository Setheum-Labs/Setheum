pragma solidity ^0.6.0;

interface ISchedule {
    event ScheduledCall(address indexed sender, address indexed contract_address, bytes task_id);
    event CanceledCall(address indexed sender, bytes task_id);
    event RescheduledCall(address indexed sender, bytes task_id);

    // Schedule call the contract.
    // Returns a boolean value indicating whether the operation succeeded.
    function scheduleCall(
        address contract_address, // The contract address to be called in future.
        uint256 value, // How much native token to send alone with the call.
        uint256 gas_limit, // The gas limit for the call. Corresponding fee will be reserved upfront and refunded after call.
        uint256 storage_limit, // The storage limit for the call. Corresponding fee will be reserved upfront and refunded after call.
        uint256 min_delay, // Minimum number of blocks before the scheduled call will be called.
        bytes calldata input_data // The input data to the call.
    )
    external
    returns (bool); // Returns a boolean value indicating whether the operation succeeded.

    // Cancel schedule call the contract.
    // Returns a boolean value indicating whether the operation succeeded.
    function cancelCall(
        bytes calldata task_id // The task id of the scheduler. Get it from the `ScheduledCall` event.
    )
    external
    returns (bool); // Returns a boolean value indicating whether the operation succeeded.

    // Reschedule call the contract.
    // Returns a boolean value indicating whether the operation succeeded.
    function rescheduleCall(
        uint256 min_delay, // Minimum number of blocks before the scheduled call will be called.
        bytes calldata task_id // The task id of the scheduler. Get it from the `ScheduledCall` event.
    )
    external
    returns (bool); // Returns a boolean value indicating whether the operation succeeded.
}
