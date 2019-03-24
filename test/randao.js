const Randao = artifacts.require('Randao');
const h = require("./helpers/helpers");

contract('Randao', (accounts) => {
  const founder = accounts[0];
  const consumer = accounts[1];
  const committer1 = accounts[2];

  let randao, bnum, campaignID, commit, commitBalkline,
    commitDeadline, commitment, deposit, secret;

  beforeEach(async () => {
    randao = await Randao.new();
  });

  it('sets the founder on deployment', async () => {
    const deployedFounder = await randao.founder.call();
    assert.equal(founder, deployedFounder);
  });

  describe('newCampaign', () => {
    context('with valid timeline and deposit', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number + 20;
        commitBalkline = 12;
        commitDeadline = 6;
        deposit = web3.utils.toWei('10', 'ether');
      });

      it('adds a new campaign', async () => {
        await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: founder, value: deposit});
        const campaigns = await randao.numCampaigns.call();
        assert.equal(campaigns.toString(), "1");
      });
    });
  });

  describe('follow', () => {
    context('with valid campaign and deposit', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number + 20;
        commitBalkline = 12;
        commitDeadline = 6;
        deposit = web3.utils.toWei('10', 'ether');
        await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: founder, value: deposit});
      });

      it('follows the campaign', async () => {
        const followed = await randao.follow.call(0, {from: consumer, value: deposit});
        assert.equal(followed, true);
      });
    });
  });

  describe('commitmentCampaign', () => {
    context('after a valid number of blocks', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number + 20;
        commitBalkline = 12;
        commitDeadline = 6;
        deposit = web3.utils.toWei('10', 'ether');
        await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: founder, value: deposit});
        await randao.follow.call(0, {from: consumer, value: deposit});
        h.mineBlocks(9);
        deposit = web3.utils.toWei('10', 'ether');
        secret = new web3.utils.BN('131242344353464564564574574567456');
      });

      it('accepts a commit', async () => {
        const web3Commitment = web3.utils.soliditySha3(secret.toString(10));
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        assert.equal(commitment, web3Commitment);
        await randao.commit(0, commitment, {from: committer1, value: deposit});
        commit = await randao.getCommitment(0, {from: committer1});
        assert.equal(commit, commitment);
      })
    });
  });

  describe('reveal', () => {
    context('after a valid number of blocks', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number + 20;
        commitBalkline = 12;
        commitDeadline = 6;
        deposit = web3.utils.toWei('10', 'ether');
        await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: founder, value: deposit});
        await randao.follow.call(0, {from: consumer, value: deposit});
        h.mineBlocks(9);
        deposit = web3.utils.toWei('10', 'ether');
        secret = new web3.utils.BN('131242344353464564564574574567456');
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        await randao.commit(0, commitment, {from: committer1, value: deposit});
        h.mineBlocks(5);
      });

      it('accepts a reveal', async () => {
        await randao.reveal(0, secret, {from: committer1});
      });
    });
  });

  describe('getRandom', () => {
    context('after a valid number of blocks', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number + 20;
        commitBalkline = 12;
        commitDeadline = 6;
        deposit = web3.utils.toWei('10', 'ether');
        await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: founder, value: deposit});
        await randao.follow.call(0, {from: consumer, value: deposit});
        h.mineBlocks(9);
        deposit = web3.utils.toWei('10', 'ether');
        secret = new web3.utils.BN('131242344353464564564574574567456');
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        await randao.commit(0, commitment, {from: committer1, value: deposit});
        h.mineBlocks(5);
        await randao.reveal(0, secret, {from: committer1});
        h.mineBlocks(5);
      });

      it('returns the random number', async () => {
        const random = await randao.getRandom.call(0, {from: consumer});
        assert.equal(random.toString(), secret.toString());
      });
    });
  });

  describe('getMyBounty', () => {
    context('after a valid campaign has ended', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number + 20;
        commitBalkline = 12;
        commitDeadline = 6;
        deposit = web3.utils.toWei('10', 'ether');
        await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: founder, value: deposit});
        await randao.follow.call(0, {from: consumer, value: deposit});
        h.mineBlocks(9);
        deposit = web3.utils.toWei('10', 'ether');
        secret = new web3.utils.BN('131242344353464564564574574567456');
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        await randao.commit(0, commitment, {from: committer1, value: deposit});
        h.mineBlocks(5);
        await randao.reveal(0, secret, {from: committer1});
        h.mineBlocks(5);
        await randao.getRandom.call(0, {from: consumer});
      });

      it('gives the bounty to committers', async () => {
        const beforeBalance = await web3.eth.getBalance(committer1);
        await randao.getMyBounty(0, {from: committer1})
        const afterBalance = await web3.eth.getBalance(committer1);
        // Commit got their initial deposit back + bounty - some gas
        assert.closeTo(+afterBalance, (+beforeBalance + +deposit + +deposit), 1200000000000000)
      });
    });
  });
});
