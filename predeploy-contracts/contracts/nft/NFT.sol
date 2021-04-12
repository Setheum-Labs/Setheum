pragma solidity ^0.6.0;

library NFT {
    function balanceOf(address account) public view returns (uint256) {
        uint256[2] memory input;

        input[0] = 0;
        input[1] = uint256(account);

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000401, input, 0x40, output, 0x20)
            ) {
                revert(0, 0)
            }
        }
        return output[0];
    }

    function ownerOf(uint256 class_id, uint256 token_id)
        public
        view
        returns (address)
    {
        uint256[3] memory input;

        input[0] = 1;
        input[1] = class_id;
        input[2] = token_id;

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000401, input, 0x60, output, 0x20)
            ) {
                revert(0, 0)
            }
        }
        return address(output[0]);
    }

    function transfer(
        address from,
        address to,
        uint256 class_id,
        uint256 token_id
    ) public view {
        uint256[5] memory input;

        input[0] = 2;
        input[1] = uint256(from);
        input[2] = uint256(to);
        input[3] = class_id;
        input[4] = token_id;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000401, input, 0xA0, 0x00, 0x00)
            ) {
                revert(0, 0)
            }
        }
    }
}
