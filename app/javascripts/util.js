
let Util = {
  mobilecheck: function() {
    if (/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent))
      return true
    else
      return false
  },

  randomNum: function(x, y, time) {
    return (Math.pow(x, 3) + Math.pow(y, 3) + Math.floor(time * 1000) + Math.floor(Math.random() * 1000)) % 62
  }
}

export default Util
