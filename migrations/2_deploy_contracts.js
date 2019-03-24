const Counter = artifacts.require("Counter");
const Randao = artifacts.require("Randao");

module.exports = function(deployer) {
  deployer.deploy(Counter);
  deployer.deploy(Randao);
};
