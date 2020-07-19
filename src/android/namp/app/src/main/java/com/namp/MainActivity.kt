package com.namp

import android.os.Bundle
import com.google.android.material.snackbar.Snackbar
import androidx.appcompat.app.AppCompatActivity
import android.view.Menu
import android.view.MenuItem

import kotlinx.android.synthetic.main.activity_main.*
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress

class MainActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        setSupportActionBar(toolbar)

        fab.setOnClickListener { view ->
            Snackbar.make(view, "Replace with your own action", Snackbar.LENGTH_LONG)
                    .setAction("Action", null).show()
        }
        val threadWithRunnable = Thread(udp_DataArrival())
        threadWithRunnable.start()
    }

    override fun onCreateOptionsMenu(menu: Menu): Boolean {
        // Inflate the menu; this adds items to the action bar if it is present.
        menuInflater.inflate(R.menu.menu_main, menu)
        return true
    }

    override fun onOptionsItemSelected(item: MenuItem): Boolean {
        // Handle action bar item clicks here. The action bar will
        // automatically handle clicks on the Home/Up button, so long
        // as you specify a parent activity in AndroidManifest.xml.
        return when (item.itemId) {
            R.id.action_settings -> true
            else -> super.onOptionsItemSelected(item)
        }
    }
}

class udp_DataArrival: Runnable {
    public override fun run() {
        println("${Thread.currentThread()} Runnable Thread Started.")
        while (true){
            receiveUDP()
        }
    }

    private fun receiveUDP() {
        val buffer = ByteArray(5000)
        var socket: DatagramSocket? = null
        try {
            socket = DatagramSocket(34254, InetAddress.getByName("0.0.0.0"))
            socket.broadcast = true
            val packet = DatagramPacket(buffer, buffer.size)
            socket.receive(packet)
            val message = String(packet.data.takeWhile { b -> b > 0 }.toByteArray())
            val (distro, deviceName, address) = message.split('|', limit = 3)
            println("$distro, $deviceName, $address")
        } catch (e: Exception) {
            println("open fun receiveUDP catch exception." + e.toString())
            e.printStackTrace()
        } finally {
            socket?.close()
        }
    }
}
