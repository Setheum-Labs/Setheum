const fs = require('fs');
const path = require('path');
const util = require('util');
const childProcess = require('child_process');
const Handlebars = require("handlebars");
const { ethers, BigNumber } = require("ethers");

const copyFile = util.promisify(fs.copyFile);
const readFile = util.promisify(fs.readFile);
const writeFile = util.promisify(fs.writeFile);
const exec = util.promisify(childProcess.exec);

// Ethereum precompiles
// 0 - 0x400
// Setheum precompiles
// 0x400 - 0x800
// Predeployed system contracts (except Mirrored ERC20)
// 0x800 - 0x1000
// Mirrored Tokens
// 0x1000000
// Mirrored NFT
// 0x2000000
// Mirrored LP Tokens
// 0x10000000000000000
const PREDEPLOY_ADDRESS_START = 0x800;

function address(start, offset) {
  const address = BigNumber.from(start).add(offset).toHexString().slice(2).padStart(40,0);
  // Returns address as a Checksum Address.
  return ethers.utils.getAddress(address);
}

const generate = async () => {
  const tokensFile = path.join(__dirname, '../resources', 'tokens.json');
  const bytecodesFile = path.join(__dirname, '../resources', 'bytecodes.json');
  const addressDir = path.join(__dirname, '../contracts/utils');

  const tokens = require(tokensFile);

  // compile to generate contracts json.
  await exec('yarn truffle-compile');

  const tokenList = tokens.reduce((output, { symbol, address }) => {
    return [...output, [symbol, ethers.utils.getAddress(address), ""]];
  }, []);

  let bytecodes = [];
  const { bytecode: token } = require(`../build/contracts/Token.json`);
  bytecodes.push(['Token', address(PREDEPLOY_ADDRESS_START, 0), token]);

  // add StateRent bytecodes
  const { bytecode: stateRent } = require(`../build/contracts/StateRent.json`);
  bytecodes.push(['StateRent', address(PREDEPLOY_ADDRESS_START, 1), stateRent]);

  // add Oracle bytecodes
  const { bytecode: oracle } = require(`../build/contracts/Oracle.json`);
  bytecodes.push(['Oracle', address(PREDEPLOY_ADDRESS_START, 2), oracle]);

  // add Schedule bytecodes
  const { bytecode: schedule } = require(`../build/contracts/Schedule.json`);
  bytecodes.push(['Schedule', address(PREDEPLOY_ADDRESS_START, 3), schedule]);

  // add DEX bytecodes
  const { bytecode: dex } = require(`../build/contracts/DEX.json`);
  bytecodes.push(['DEX', address(PREDEPLOY_ADDRESS_START, 4), dex]);

  // merge tokenList into bytecodes
  bytecodes = tokenList.concat(bytecodes);

  await writeFile(bytecodesFile, JSON.stringify(bytecodes, null, 2), 'utf8');

  // generate address constant for sol
  let tmpl = fs.readFileSync(path.resolve(__dirname, '../resources', 'address.sol.hbs'), 'utf8');
  let template = Handlebars.compile(tmpl);
  await writeFile(path.join(addressDir, 'Address.sol'), template(bytecodes), 'utf8');

  // generate address constant for js
  tmpl = fs.readFileSync(path.resolve(__dirname, '../resources', 'address.js.hbs'), 'utf8');
  template = Handlebars.compile(tmpl);
  await writeFile(path.join(addressDir, 'Address.js'), template(bytecodes), 'utf8');

  // recompile Address.sol
  await exec('yarn truffle-compile');

  // generate Address.d.ts
  await exec('tsc contracts/utils/Address.js --declaration --allowJs --emitDeclarationOnly');
};

const main = async () => {
  try {
    await generate();
  } catch (err) {
    console.log('>>> generating contracts bytecode failed: ', err);
  }
};

main();
