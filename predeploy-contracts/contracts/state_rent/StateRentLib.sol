pragma solidity ^0.6.0;

library StateRentLib {
    function newContractExtraBytes() internal view returns (uint256) {
        uint256[1] memory input;

        input[0] = 0;

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000402, input, 0x20, output, 0x20)
            ) {
                revert(0, 0)
            }
        }
        return output[0];
    }

    function storageDepositPerByte() internal view returns (uint256) {
        uint256[1] memory input;

        input[0] = 1;

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000402, input, 0x20, output, 0x20)
            ) {
                revert(0, 0)
            }
        }
        return output[0];
    }

    function maintainerOf(address contract_address)
        internal
        view
        returns (address)
    {
        uint256[2] memory input;

        input[0] = 2;
        input[1] = uint256(contract_address);

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000402, input, 0x40, output, 0x20)
            ) {
                revert(0, 0)
            }
        }
        return address(output[0]);
    }

    function developerDeposit() internal view returns (uint256) {
        uint256[1] memory input;

        input[0] = 3;

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000402, input, 0x20, output, 0x20)
            ) {
                revert(0, 0)
            }
        }
        return output[0];
    }

    function deploymentFee() internal view returns (uint256) {
        uint256[1] memory input;

        input[0] = 4;

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000402, input, 0x20, output, 0x20)
            ) {
                revert(0, 0)
            }
        }
        return output[0];
    }

    function transferMaintainer(
        address sender,
        address contract_address,
        address new_maintainer
    ) internal view {
        uint256[4] memory input;

        input[0] = 128;
        input[1] = uint256(sender);
        input[2] = uint256(contract_address);
        input[3] = uint256(new_maintainer);

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000402, input, 0x80, 0x00, 0x00)
            ) {
                revert(0, 0)
            }
        }
    }
}

