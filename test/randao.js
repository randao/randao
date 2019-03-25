const Randao = artifacts.require('Randao');
const h = require("./helpers/helpers");

contract('Randao', (accounts) => {
  const founder = accounts[0];
  const consumer = accounts[1];
  const follower1 = accounts[2];
  const committer1 = accounts[3];

  let deposit = web3.utils.toWei('10', 'ether');

  let randao, bnum, campaignID, commit, commitBalkline,
    commitDeadline, commitment, secret;

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

      it('adds a new campaign with a bounty', async () => {
        await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: consumer, value: deposit});
        const campaigns = await randao.numCampaigns.call();
        assert.equal(campaigns.toString(), "1");
      });

      it('adds a new campaign without a bounty', async () => {
        await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: consumer, value: 0});
        const campaigns = await randao.numCampaigns.call();
        assert.equal(campaigns.toString(), "1");
      });
    });

    context('when the given bnum equals the current blocknumber', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number;
        commitBalkline = 12;
        commitDeadline = 6;
        deposit = web3.utils.toWei('10', 'ether');
      });

      it('does not add a new campaign', async () => {
        await h.assertThrowsAsync(async () => {
          await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: consumer, value: deposit});
        }, '');
        const campaigns = await randao.numCampaigns.call();
        assert.equal(campaigns.toString(), "0");
      });
    });

    context('when the given commitDeadline is less than commitBalkline', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number + 20;
        commitBalkline = 6;
        commitDeadline = 12;
        deposit = web3.utils.toWei('10', 'ether');
      });

      it('does not add a new campaign', async () => {
        await h.assertThrowsAsync(async () => {
          await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: consumer, value: deposit});
        }, '');
        const campaigns = await randao.numCampaigns.call();
        assert.equal(campaigns.toString(), "0");
      });
    });

    context('when the given bnum and commitBalkline is less than the blocknumber', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number + 10;
        commitBalkline = 12;
        commitDeadline = 6;
        deposit = web3.utils.toWei('10', 'ether');
      });

      it('does not add a new campaign', async () => {
        await h.assertThrowsAsync(async () => {
          await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: consumer, value: deposit});
        }, '');
        const campaigns = await randao.numCampaigns.call();
        assert.equal(campaigns.toString(), "0");
      });
    });

    context('when the deposit is 0', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number + 20;
        commitBalkline = 12;
        commitDeadline = 6;
        deposit = 0;
      });

      it('does not add a new campaign', async () => {
        await h.assertThrowsAsync(async () => {
          await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: consumer, value: deposit});
        }, '');
        const campaigns = await randao.numCampaigns.call();
        assert.equal(campaigns.toString(), "0");
      });
    });

  });

  describe('follow', () => {
    context('with valid campaign and deposit', () => {
      beforeEach(async () => {
        await h.setupNewCampaign(randao, consumer);
      });

      it('follows the campaign with value added for bounty', async () => {
        const followed = await randao.follow.call(0, {from: follower1, value: deposit});
        assert.equal(followed, true);
      });

      it('follows the campaign without value added for bounty', async () => {
        const followed = await randao.follow.call(0, {from: follower1, value: 0});
        assert.equal(followed, true);
      });
    });

    context('if a campaign does not exist', () => {
      it('has nothing to follow', async () => {
        await h.assertThrowsAsync(async () => {
          await randao.follow.call(0, {from: follower1, value: 0});
        }, '');
      });
    });
  });

  describe('commitmentCampaign', () => {
    context('before the commit phase', () => {
      beforeEach(async () => {
        await h.setupNewCampaign(randao, consumer);
        await randao.follow.call(0, {from: follower1, value: deposit});
        secret = new web3.utils.BN('131242344353464564564574574567456');
      });

      it('does not accept commits', async () => {
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        await h.assertThrowsAsync(async () => {
          await randao.commit(0, commitment, {from: committer1, value: deposit});
        }, '');
      });
    });

    context('during the commit phase', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number + 20;
        commitBalkline = 12;
        commitDeadline = 6;
        deposit = web3.utils.toWei('10', 'ether');
        await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: consumer, value: deposit});
        await randao.follow.call(0, {from: follower1, value: deposit});
        h.mineBlocks(9);
        secret = new web3.utils.BN('131242344353464564564574574567456');
      });

      it('accepts a commit', async () => {
        const web3Commitment = web3.utils.soliditySha3(secret.toString(10));
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        assert.equal(commitment, web3Commitment);
        await randao.commit(0, commitment, {from: committer1, value: deposit});
        commit = await randao.getCommitment(0, {from: committer1});
        assert.equal(commit, commitment);
      });

      it('does not accept an empty commit', async () => {
        await h.assertThrowsAsync(async () => {
          await randao.commit(0, 0x0, {from: committer1, value: deposit});
        }, '');
      });

      it('does not accept 0 deposit', async () => {
        await h.assertThrowsAsync(async () => {
          await randao.commit(0, commitment, {from: committer1, value: 0});
        }, '');
      });
    });

    context('after the commit phase', () => {
      beforeEach(async () => {
        await h.setupNewCampaign(randao, consumer);
        await randao.follow.call(0, {from: follower1, value: deposit});
        h.mineBlocks(19);
        secret = new web3.utils.BN('131242344353464564564574574567456');
      });

      it('does not accept commits', async () => {
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        await h.assertThrowsAsync(async () => {
          await randao.commit(0, commitment, {from: committer1, value: deposit});
        }, '');
      });
    });
  });

  describe('reveal', () => {
    context('before the reveal phase', () => {
      beforeEach(async () => {
        await h.setupNewCampaign(randao, consumer);
        await randao.follow.call(0, {from: follower1, value: deposit});
        h.mineBlocks(9);
        secret = new web3.utils.BN('131242344353464564564574574567456');
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        await randao.commit(0, commitment, {from: committer1, value: deposit});
      });

      it('does not accept reveals', async () => {
        await h.assertThrowsAsync(async () => {
          await randao.reveal(0, secret, {from: committer1});
        }, '');
      });

    });
    context('during the reveal phase', () => {
      beforeEach(async () => {
        await h.setupNewCampaign(randao, consumer);
        await randao.follow.call(0, {from: follower1, value: deposit});
        h.mineBlocks(9);
        secret = new web3.utils.BN('131242344353464564564574574567456');
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        await randao.commit(0, commitment, {from: committer1, value: deposit});
        h.mineBlocks(5);
      });

      it('accepts a reveal', async () => {
        await randao.reveal(0, secret, {from: committer1});
      });
    });

    context('after the reveal phase', () => {
      beforeEach(async () => {
        await h.setupNewCampaign(randao, consumer);
        await randao.follow.call(0, {from: follower1, value: deposit});
        h.mineBlocks(9);
        secret = new web3.utils.BN('131242344353464564564574574567456');
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        await randao.commit(0, commitment, {from: committer1, value: deposit});
        h.mineBlocks(15);
      });

      it('does not accept reveals', async () => {
        await h.assertThrowsAsync(async () => {
          await randao.reveal(0, secret, {from: committer1});
        }, '');
      });
    });
  });

  describe('getRandom', () => {
    context('before the bounty phase', () => {
      beforeEach(async () => {
        await h.setupNewCampaign(randao, consumer);
        await randao.follow.call(0, {from: follower1, value: deposit});
        h.mineBlocks(9);
        secret = new web3.utils.BN('131242344353464564564574574567456');
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        await randao.commit(0, commitment, {from: committer1, value: deposit});
        h.mineBlocks(5);
        await randao.reveal(0, secret, {from: committer1});
      });

      it('does not return the random number', async () => {
        await h.assertThrowsAsync(async () => {
          await randao.getRandom.call(0, {from: consumer});
        }, '');
      });
    });

    context('during the bounty phase', () => {
      beforeEach(async () => {
        await h.setupNewCampaign(randao, consumer);
        await randao.follow.call(0, {from: follower1, value: deposit});
        h.mineBlocks(9);
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

      it('returns the random number after the bounty phase', async () => {
        h.mineBlocks(15);
        const random = await randao.getRandom.call(0, {from: consumer});
        assert.equal(random.toString(), secret.toString());
      });
    });
  });

  describe('getMyBounty', () => {
    context('after a valid campaign has ended', () => {
      beforeEach(async () => {
        await h.setupNewCampaign(randao, consumer);
        await randao.follow.call(0, {from: follower1, value: deposit});
        h.mineBlocks(9);
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
        // Committer got their initial deposit back + bounty - some gas
        assert.closeTo(+afterBalance, (+beforeBalance + +deposit + +deposit), 1200000000000000)
      });
    });
  });

  describe('refundBounty', () => {
    context('after a campaign has ended without revealing', () => {
      beforeEach(async () => {
        await h.setupNewCampaign(randao, consumer);
        await randao.follow.call(0, {from: follower1, value: deposit});
        h.mineBlocks(9);
        secret = new web3.utils.BN('131242344353464564564574574567456');
        commitment = await randao.shaCommit(secret.toString(10), {from: committer1});
        await randao.commit(0, commitment, {from: committer1, value: deposit});
        h.mineBlocks(8);
      });

      it('refunds the consumer', async () => {
        const beforeBalance = await web3.eth.getBalance(consumer);
        await randao.refundBounty(0, {from: consumer});
        const afterBalance = await web3.eth.getBalance(consumer);
        // Consumer got their initial deposit back - some gas
        assert.closeTo(+afterBalance, (+beforeBalance + +deposit), 500000000000000);
      });

      it('refunds the committer', async () => {
        const beforeBalance = await web3.eth.getBalance(committer1);
        await randao.getMyBounty(0, {from: committer1});
        const afterBalance = await web3.eth.getBalance(committer1);
        // Committer got their initial deposit back - some gas
        assert.closeTo(+afterBalance, (+beforeBalance + +deposit), 1200000000000000);
      });

      it('refunds the follower', async () => {
        const beforeBalance = await web3.eth.getBalance(follower1);
        await randao.getMyBounty(0, {from: follower1});
        const afterBalance = await web3.eth.getBalance(follower1);
        // Follower got their initial deposit back - some gas
        assert.closeTo(+afterBalance, (+beforeBalance + +deposit), 1200000000000000);
      });
    });
  });
});
