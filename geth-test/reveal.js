miner.start(2); admin.sleepBlocks(2); miner.stop();

console.log('reveal plz wait....');

randao.reveal.sendTransaction(campaignID, secret, {from: eth.accounts[2], value: web3.toWei('10', 'ether') })

miner.start(1); admin.sleepBlocks(1); miner.stop();

