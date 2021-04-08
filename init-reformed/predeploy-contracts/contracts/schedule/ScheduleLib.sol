pragma solidity ^0.6.0;

library ScheduleCallLib {
    function scheduleCall(
        address sender,
        address contract_address,
        uint256 value,
        uint256 gas_limit,
        uint256 storage_limit,
        uint256 min_delay,
        bytes memory input_data
    ) internal view returns (bytes memory) {
        uint input_data_capacity = (input_data.length + 31)/32;
        // param + input_len + input_data
        uint input_size = 7 + 1 + input_data_capacity;

        // Dynamic arrays will add the array size to the front of the array, pre-compile needs to deal with the `size`.
        uint256[] memory input = new uint256[](input_size);

        input[0] = 0;
        input[1] = uint256(sender);
        input[2] = uint256(contract_address);
        input[3] = uint256(value);
        input[4] = uint256(gas_limit);
        input[5] = uint256(storage_limit);
        input[6] = uint256(min_delay);

        // input_len
        input[7] = uint256(input_data.length);

        for (uint i = 0; i < input_data_capacity; i++) {
            input[8 + i] = bytes2Uint(input_data, i);
        }

        // Dynamic arrays will add the array size to the front of the array, so need extra 1 size.
        uint input_size_32 = (input_size + 1) * 32;

        uint256[3] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000404, input, input_size_32, output, 0x60)
            ) {
                revert(0, 0)
            }
        }

        bytes memory task_id = new bytes(output[0]);
        bytes memory result = abi.encodePacked(output[1], output[2]);
        for (uint i = 0; i < task_id.length; i++) {
            task_id[i] = result[i];
        }

        return task_id;
    }

    function cancelCall(
        address sender,
        bytes memory task_id
    ) internal {
        uint input_data_capacity = (task_id.length + 31)/32;
        // param + task_id_len + task_id
        uint input_size = 2 + 1 + input_data_capacity;
        uint256[] memory input = new uint256[](input_size);

        input[0] = 1;
        input[1] = uint256(sender);

        // task_id_len
        input[2] = uint256(task_id.length);

        for (uint i = 0; i < input_data_capacity; i++) {
            input[3 + i] = bytes2Uint(task_id, i);
        }

        // Dynamic arrays will add the array size to the front of the array, so need extra 1 size.
        uint input_size_32 = (input_size + 1) * 32;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000404, input, input_size_32, 0x00, 0x00)
            ) {
                revert(0, 0)
            }
        }
    }

    function rescheduleCall(
        address sender,
        uint256 min_delay,
        bytes memory task_id
    ) internal {
        uint input_data_capacity = (task_id.length + 31)/32;
        // param + task_id_len + task_id
        uint input_size = 3 + 1 + input_data_capacity;
        uint256[] memory input = new uint256[](input_size);

        input[0] = 2;
        input[1] = uint256(sender);
        input[2] = uint256(min_delay);

        // task_id_len
        input[3] = uint256(task_id.length);

        for (uint i = 0; i < input_data_capacity; i++) {
            input[4 + i] = bytes2Uint(task_id, i);
        }

        // Dynamic arrays will add the array size to the front of the array, so need extra 1 size.
        uint input_size_32 = (input_size + 1) * 32;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000404, input, input_size_32, 0x00, 0x00)
            ) {
                revert(0, 0)
            }
        }
    }

    function bytes2Uint(bytes memory bs, uint index) public pure returns (uint) {
        // require(bs.length >= start + 32, "slicing out of range");
        // if bs.length < start + 32, 0 will be added at the end.
        uint start = index * 32;
        uint x;
        assembly {
            x := mload(add(bs, add(0x20, start)))
        }
        return x;
    }
}

