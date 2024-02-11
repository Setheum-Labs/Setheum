// SPDX-License-Identifier: GPL-3.0-or-later

pragma solidity ^0.8.0;

import "./IOracle.sol";

contract Oracle is IOracle {
    address constant private precompile = address(0x0000000000000000000000000000000000000403);

    /**
     * @dev Get the price of the currency_id.
     * Returns the (price, timestamp)
     */
    function getPrice(address token)
    public
    view
    override
    returns (uint256)
    {
        require(token != address(0), "Oracle: token is zero address");

        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodeWithSignature("getPrice(address)", token));
        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (uint256));
    }
}
