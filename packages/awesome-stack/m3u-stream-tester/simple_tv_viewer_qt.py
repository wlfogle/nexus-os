import sys
from PyQt5 import QtWidgets, QtGui, QtCore
import vlc

class MediaPlayer(QtWidgets.QMainWindow):
    def __init__(self):
        super(MediaPlayer, self).__init__()
        self.setWindowTitle("HDHomeRun TV Viewer")
        self.setGeometry(100, 100, 800, 600)

        self.widget = QtWidgets.QWidget(self)
        self.setCentralWidget(self.widget)

        self.vboxLayout = QtWidgets.QVBoxLayout()
        self.widget.setLayout(self.vboxLayout)

        # Video frame
        self.videoFrame = QtWidgets.QFrame()
        self.vboxLayout.addWidget(self.videoFrame)
        self.palette = self.videoFrame.palette()
        self.palette.setColor(QtGui.QPalette.Window, QtGui.QColor(0, 0, 0))
        self.videoFrame.setPalette(self.palette)
        self.videoFrame.setAutoFillBackground(True)

        # VLC player instance
        self.instance = vlc.Instance()
        self.mediaplayer = self.instance.media_player_new()

        # Control buttons
        self.hboxLayout = QtWidgets.QHBoxLayout()

        self.playButton = QtWidgets.QPushButton("Play")
        self.playButton.clicked.connect(self.play_pause)
        self.hboxLayout.addWidget(self.playButton)

        self.volumeSlider = QtWidgets.QSlider(QtCore.Qt.Horizontal)
        self.volumeSlider.setMaximum(200)
        self.volumeSlider.setValue(self.mediaplayer.audio_get_volume())
        self.volumeSlider.sliderMoved.connect(self.set_volume)
        self.hboxLayout.addWidget(self.volumeSlider)

        self.vboxLayout.addLayout(self.hboxLayout)

        self.isPaused = False


    def play_pause(self):
        if self.mediaplayer.is_playing():
            self.mediaplayer.pause()
            self.playButton.setText("Play")
            self.isPaused = True
        else:
            if self.isPaused:
                self.mediaplayer.play()
                self.playButton.setText("Pause")
            else:
                # Create a new media
                media = self.instance.media_new("http://192.168.12.215:5004/auto/v11.1")
                self.mediaplayer.set_media(media)
                self.mediaplayer.set_xwindow(self.videoFrame.winId())
                self.mediaplayer.play()
                self.playButton.setText("Pause")

    def set_volume(self, value):
        self.mediaplayer.audio_set_volume(value)

if __name__ == "__main__":
    app = QtWidgets.QApplication(sys.argv)
    player = MediaPlayer()
    player.show()
    sys.exit(app.exec_())

