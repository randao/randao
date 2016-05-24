personal.unlockAccount(web3.eth.accounts[0], "Write here a good, randomly generated, passphrase!")
personal.unlockAccount(web3.eth.accounts[1], "Write here a good, randomly generated, passphrase!")
personal.unlockAccount(web3.eth.accounts[2], "Write here a good, randomly generated, passphrase!")
personal.unlockAccount(web3.eth.accounts[3], "Write here a good, randomly generated, passphrase!")

var val_cont = web3.toWei(10, "wei")    //to be sure we have enough for val + gas

console.log('blockNumber: ', web3.eth.blockNumber);
console.log('target_block: ', target_block);
console.log('newCampaign plz wait....');
randao.newCampaign.sendTransaction(target_block, deposit, 6, 12, {from:eth.accounts[1],gas:100000,value:val_cont})

miner.start(1); admin.sleepBlocks(1); miner.stop();

var campaignID = randao.numCampaigns.call();

console.log('campaignID: ', campaignID);
console.log('campaigns: ', randao.campaigns.call());
