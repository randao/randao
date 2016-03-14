console.log('mining and get random wait....');

miner.start(2); admin.sleepBlocks(2); miner.stop();

var random = randao.getRandom.call(campaignID);

console.log('Congratulation..', random);

