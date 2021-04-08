// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.6.0;

library MultiCurrency {
    function totalSupply(uint256 currencyId) internal view returns (uint256) {
        uint256[2] memory input;

        input[0] = 0;
        input[1] = currencyId;

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000400, input, 0x40, output, 0x20)
            ) {
                revert(0, 0)
            }
        }

        return output[0];
    }

    function balanceOf(uint256 currencyId, address account) internal view returns (uint256) {
        uint256[3] memory input;

        input[0] = 1;
        input[1] = currencyId;
        input[2] = uint256(account);

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000400, input, 0x60, output, 0x20)
            ) {
                revert(0, 0)
            }
        }

        return output[0];
    }

    function transfer(uint256 currencyId, address sender, address recipient, uint256 amount) internal view {
        uint256[5] memory input;

        input[0] = 2;
        input[1] = currencyId;
        input[2] = uint256(sender);
        input[3] = uint256(recipient);
        input[4] = amount;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000400, input, 0xA0, 0x00, 0x00)
            ) {
                revert(0, 0)
            }
        }
    }
}
