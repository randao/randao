import Util from './util'
import Progress from './progress'

let mouseTracker = {
  "max": 30,
  "count": 0,
  "string": "Your address: ",
  "generating": false,
  "mouseInside": false,
  "chars": "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",

  init: function() {
    this.generating = true
    Progress.init()

    $(document).on("mousemove", '#randao-box', e => {
      this.mouseInside = true
      this.move(e)
    })

    $(document).on("mouseleave", '#randao-box', e => {
      this.mouseInside = false
    })
  },

  move: function(position) {
    if (!this.mouseInside || !this.generating) return false

    let X = position.pageX
    let Y = position.pageY
    this.count++

    if (position.target.className == 'randao-box') {
      let time = new Date().getTime()

      if (time % 5 == 1) {
        let tapDiv = $('<div>')

        tapDiv.addClass("tap").css({
          left: X,
          top: Y
        }).appendTo("body").fadeOut(1000)
      }
    }

    if (this.count % 10 == 1) {
      if (this.max--) {
        let num = Util.randomNum(X, Y, new Date().getTime())

        this.string += this.chars.charAt(num % this.chars.length)
        $("#code").html(this.string)

        Progress.growth()
      } else {
        $('#code').addClass('active')
        $('.send-address').addClass('show')
        this.generating = false
      }
    }
  }
}

export default mouseTracker
