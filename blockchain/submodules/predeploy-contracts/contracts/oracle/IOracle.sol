// SPDX-License-Identifier: GPL-3.0-or-later

pragma solidity ^0.8.0;

interface IOracle {
    // Get the price of the currency_id.
    // Returns the price.
    function getPrice(address token) external view returns (uint256);
}
