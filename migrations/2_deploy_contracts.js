const Randao = artifacts.require("Randao");

module.exports = function(deployer) {
  deployer.deploy(Randao);
};
