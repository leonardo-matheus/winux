package com.winux.connect.protocol

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.*
import javax.crypto.Cipher
import javax.crypto.KeyAgreement
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Encryption handler for secure communication
 * Uses ECDH key exchange and AES-GCM for encryption
 */
@Singleton
class Encryption @Inject constructor() {

    private var keyPair: KeyPair? = null
    private var sharedSecret: SecretKey? = null
    private var peerPublicKey: PublicKey? = null

    companion object {
        private const val KEY_ALIAS = "winux_connect_key"
        private const val ANDROID_KEYSTORE = "AndroidKeyStore"
        private const val EC_ALGORITHM = "EC"
        private const val KEY_SIZE = 256
        private const val AES_ALGORITHM = "AES/GCM/NoPadding"
        private const val GCM_TAG_LENGTH = 128
        private const val GCM_IV_LENGTH = 12
    }

    /**
     * Check if encryption keys exist
     */
    fun hasKeys(): Boolean = keyPair != null

    /**
     * Check if encryption is enabled (shared secret established)
     */
    fun isEnabled(): Boolean = sharedSecret != null

    /**
     * Generate EC key pair
     */
    fun generateKeyPair() {
        try {
            val keyPairGenerator = KeyPairGenerator.getInstance(EC_ALGORITHM)
            keyPairGenerator.initialize(KEY_SIZE, SecureRandom())
            keyPair = keyPairGenerator.generateKeyPair()
        } catch (e: Exception) {
            throw SecurityException("Failed to generate key pair", e)
        }
    }

    /**
     * Get public key as Base64 string
     */
    fun getPublicKey(): String {
        val publicKey = keyPair?.public
            ?: throw IllegalStateException("Key pair not generated")

        return Base64.encodeToString(publicKey.encoded, Base64.NO_WRAP)
    }

    /**
     * Set peer's public key and derive shared secret
     */
    fun setPeerPublicKey(base64Key: String) {
        try {
            val keyBytes = Base64.decode(base64Key, Base64.NO_WRAP)

            val keyFactory = KeyFactory.getInstance(EC_ALGORITHM)
            val keySpec = java.security.spec.X509EncodedKeySpec(keyBytes)
            peerPublicKey = keyFactory.generatePublic(keySpec)

            deriveSharedSecret()
        } catch (e: Exception) {
            throw SecurityException("Failed to set peer public key", e)
        }
    }

    /**
     * Derive shared secret using ECDH
     */
    private fun deriveSharedSecret() {
        val privateKey = keyPair?.private
            ?: throw IllegalStateException("Key pair not generated")
        val peerKey = peerPublicKey
            ?: throw IllegalStateException("Peer public key not set")

        try {
            val keyAgreement = KeyAgreement.getInstance("ECDH")
            keyAgreement.init(privateKey)
            keyAgreement.doPhase(peerKey, true)

            val secret = keyAgreement.generateSecret()

            // Use SHA-256 to derive AES key from shared secret
            val digest = MessageDigest.getInstance("SHA-256")
            val hash = digest.digest(secret)
            sharedSecret = SecretKeySpec(hash, "AES")
        } catch (e: Exception) {
            throw SecurityException("Failed to derive shared secret", e)
        }
    }

    /**
     * Encrypt a message
     */
    fun encrypt(plaintext: String): String {
        val secret = sharedSecret
            ?: throw IllegalStateException("Shared secret not established")

        try {
            val cipher = Cipher.getInstance(AES_ALGORITHM)

            // Generate random IV
            val iv = ByteArray(GCM_IV_LENGTH)
            SecureRandom().nextBytes(iv)

            val spec = GCMParameterSpec(GCM_TAG_LENGTH, iv)
            cipher.init(Cipher.ENCRYPT_MODE, secret, spec)

            val ciphertext = cipher.doFinal(plaintext.toByteArray(Charsets.UTF_8))

            // Combine IV and ciphertext
            val combined = ByteArray(iv.size + ciphertext.size)
            System.arraycopy(iv, 0, combined, 0, iv.size)
            System.arraycopy(ciphertext, 0, combined, iv.size, ciphertext.size)

            return Base64.encodeToString(combined, Base64.NO_WRAP)
        } catch (e: Exception) {
            throw SecurityException("Encryption failed", e)
        }
    }

    /**
     * Decrypt a message
     */
    fun decrypt(encryptedBase64: String): String {
        val secret = sharedSecret
            ?: throw IllegalStateException("Shared secret not established")

        try {
            val combined = Base64.decode(encryptedBase64, Base64.NO_WRAP)

            // Extract IV and ciphertext
            val iv = combined.copyOfRange(0, GCM_IV_LENGTH)
            val ciphertext = combined.copyOfRange(GCM_IV_LENGTH, combined.size)

            val cipher = Cipher.getInstance(AES_ALGORITHM)
            val spec = GCMParameterSpec(GCM_TAG_LENGTH, iv)
            cipher.init(Cipher.DECRYPT_MODE, secret, spec)

            val plaintext = cipher.doFinal(ciphertext)
            return String(plaintext, Charsets.UTF_8)
        } catch (e: Exception) {
            throw SecurityException("Decryption failed", e)
        }
    }

    /**
     * Clear all keys and secrets
     */
    fun clear() {
        keyPair = null
        peerPublicKey = null
        sharedSecret = null
    }
}
