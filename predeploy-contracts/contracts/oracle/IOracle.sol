pragma solidity ^0.6.0;

interface IOracle {
    // Get the price of the currency_id.
    // Returns the price.
    function getPrice(address token) external view returns (uint256);
}
