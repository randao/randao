miner.start(1); admin.sleepBlocks(1); miner.stop();

console.log('reveal plz wait...');
console.log('reveal at blockNumber: ', web3.eth.blockNumber);

randao.reveal.sendTransaction(campaignID, secret, {from: eth.accounts[2], value: web3.toWei('10', 'ether') })

miner.start(1); admin.sleepBlocks(1); miner.stop();
console.log('campaigns: ', randao.campaigns.call());
