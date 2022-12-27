const SIGNAL_TYPE_JOIN = "join";
const SIGNAL_TYPE_LEAVE = "leave";
const SIGNAL_TYPE_NEW_PEER = "new-peer";
const SIGNAL_TYPE_PEER_LEAVE = "peer-leave";
const SIGNAL_TYPE_OFFER = "offer";
const SIGNAL_TYPE_ANSWER = "answer";
const SIGNAL_TYPE_ICE_CANDIDATE = "ice-candidate";

const USER_TYPE_SENDER = "sender";
const USER_TYPE_RECEIVER = "receiver";

var userId = Math.random().toString(36).substring(2, 9);

class RTCEngine {
  init() {
    this._peerConnection = null;
    this.peerId = null;
    this._onIceCandidate = this._onIceCandidate.bind(this);
    this.recvDataChannel = this.recvDataChannel.bind(this);
  }

  _createPeerConnection() {
    if (this._peerConnection != null) {
      return;
    }
    this._peerConnection = new RTCPeerConnection();
    this._peerConnection.onicecandidate = this._onIceCandidate;
    this._peerConnection.onconnectionstatechange = (event) => {
      if (this._peerConnection != null) {
        console.log("connectionState: " + this._peerConnection.connectionState);
      }
    };
    this._peerConnection.oniceconnectionstatechange = (event) => {
      if (this._peerConnection != null) {
        console.log(
          "iceConnectionState: " + this._peerConnection.iceConnectionState
        );
      }
    };
    this._peerConnection.ontrack = (event) => {
      console.log("on track", event);
      if (this.type === USER_TYPE_SENDER) {
        return;
      }
      this.remoteVideo.srcObject = event.streams[0];
      this.remoteStream = event.streams[0];
    };
    this.localStream?.getTracks().forEach((track) => {
      this._peerConnection.addTrack(track, this.localStream);
    });
    // if (this.type === USER_TYPE_SENDER) {
    this.recvDataChannel();
    // }
  }

  _onWsMessage(event) {
    var signal = JSON.parse(event.data);
    switch (signal.type) {
      case SIGNAL_TYPE_NEW_PEER:
        if (this.type === USER_TYPE_SENDER) {
          this._onNewPeer(signal);
        }
        break;
      case SIGNAL_TYPE_PEER_LEAVE:
        if (this.type === USER_TYPE_SENDER) {
          this._onPeerLeave(signal);
        }
        break;
      case SIGNAL_TYPE_OFFER:
        if (this.type === USER_TYPE_RECEIVER) {
          this._onRemoteOffer(signal);
        }
        break;
      case SIGNAL_TYPE_ANSWER:
        if (this.type === USER_TYPE_SENDER) {
          this._onRemoteAnswer(signal);
        }
        break;
      case SIGNAL_TYPE_ICE_CANDIDATE:
        this._onRemoteIceCandidate(signal);
        break;
    }
  }

  initSender(wsUrl, localStream) {
    this.type = USER_TYPE_SENDER;
    this.localStream = localStream;
    this._initWebSockets(wsUrl);
  }

  initReceiver(wsUrl, remoteVideo) {
    this.type = USER_TYPE_RECEIVER;
    this.remoteVideo = remoteVideo;
    this.remoteStream = null;
    this._initWebSockets(wsUrl);
  }

  _initWebSockets(wsUrl) {
    this._ws = new WebSocket(wsUrl);
    this._ws.onopen = () => {
      console.log("ws open user id:", userId);
      this._ws.send(
        JSON.stringify({
          type: SIGNAL_TYPE_JOIN,
          userId: userId,
        })
      );
    };
    this._ws.onmessage = this._onWsMessage.bind(this);
  }
  // new peer join, create offer
  _onNewPeer(signal) {
    console.log("new-peer", signal);
    this.peerId = signal.peerId;
    this._createPeerConnection();
    this._peerConnection
      .createOffer()
      .then((offer) => {
        this._peerConnection.setLocalDescription(offer);
        console.log("got offer", offer);
        return offer;
      })
      .then((offer) => {
        this._ws.send(
          JSON.stringify({
            type: SIGNAL_TYPE_OFFER,
            userId: userId,
            peerId: signal.peerId,
            offer: offer,
          })
        );
      })
      .catch(console.error);
  }
  // peer has leave, reset peer connection
  _onPeerLeave(signal) {
    console.log("peer-leave", signal);
    this._peerConnection.close();
    if (this.remoteVideo) {
      this.remoteStream.getTracks().forEach((track) => {
        track.stop();
      });
      this.remoteVideo.srcObject = null;
      this.remoteStream = null;
    }
  }
  // got remote offer
  // set remote description and send answer
  _onRemoteOffer(message) {
    console.log("remote offer ", message);
    this.peerId = message.userId;
    this._createPeerConnection();
    this._peerConnection
      .setRemoteDescription(new RTCSessionDescription(message.offer))
      .then(async () => {
        // doAnswer
        let answer = await this._peerConnection.createAnswer();
        await this._peerConnection.setLocalDescription(answer);
        var answerInfo = {
          type: SIGNAL_TYPE_ANSWER,
          userId: userId,
          peerId: this.peerId,
          answer: this._peerConnection.localDescription,
        };
        this._ws.send(JSON.stringify(answerInfo));
        console.log("send answer", answerInfo);
      })
      .catch(console.error);
  }
  // remote answer
  _onRemoteAnswer(message) {
    console.log("remote answer ", message);
    this._peerConnection
      .setRemoteDescription(new RTCSessionDescription(message.answer))
      .catch(console.error);
  }
  // local ice candidate
  _onIceCandidate(event) {
    console.log("on ice candidate", event);
    if (!event.candidate) return;
    this._ws.send(
      JSON.stringify({
        type: SIGNAL_TYPE_ICE_CANDIDATE,
        userId: userId,
        peerId: this.peerId,
        candidate: event.candidate,
      })
    );
  }
  // remote ice candidate
  _onRemoteIceCandidate(event) {
    console.log("on remote ice candidate", event);
    if (!event.candidate) return;
    this._createPeerConnection();
    this._peerConnection.addIceCandidate(event.candidate);
  }

  createDataChannel(label) {
    this._createPeerConnection();
    return this._peerConnection.createDataChannel(label);
  }

  recvDataChannel() {
    this._peerConnection.ondatachannel = (event) => {
      console.log("on data channel", event);
      const channel = event.channel;
      channel.onopen = (event) => {
        console.log("channel open", event);
      };
      channel.onmessage = this.onDataChannelMessage;
    };
  }
}

// export default RTCEngine;
