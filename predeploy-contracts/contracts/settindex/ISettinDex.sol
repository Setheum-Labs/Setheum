pragma solidity ^0.6.0;

interface ISettinDex {
    event Swaped(address indexed sender, address[] path, uint256 supply_amount, uint256 target_amount);

    // Get liquidity pool of the currency_id_a and currency_id_b.
    // Returns (liquidity_a, liquidity_b)
    function getLiquidityPool(address tokenA, address tokenB) external view returns (uint256, uint256);

    // Get swap target amount.
    // Returns (target_amount)
    function getSwapTargetAmount(address[] calldata path, uint256 supplyAmount) external view returns (uint256);

    // Get swap supply amount.
    // Returns (supply_amount)
    function getSwapSupplyAmount(address[] calldata path, uint256 targetAmount) external view returns (uint256);

    // Swap with exact supply.
    // Returns a boolean value indicating whether the operation succeeded.
    function swapWithExactSupply(address[] calldata path, uint256 supplyAmount, uint256 minTargetAmount) external returns (bool);

    // Swap with exact target.
    // Returns a boolean value indicating whether the operation succeeded.
    function swapWithExactTarget(address[] calldata path, uint256 targetAmount, uint256 maxSupplyAmount) external returns (bool);
}
