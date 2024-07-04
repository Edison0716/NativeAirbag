package com.edisonlivings.airbag


class AirbagConfig {
    private var signalConfig: MutableMap<Int, Map<String, Set<String>>> ?= null


    fun getSignalConfig(): MutableMap<Int, Map<String, Set<String>>>? {
        return signalConfig
    }

    class Builder {
        private val signalConfig: MutableMap<Int, Map<String, Set<String>>> = mutableMapOf()

        /**
         * if set backtrace, will only cover this backtrace.
         */
        fun addSignalConfig(maskSignal: Int, elfName: String, backtrace:String): Builder {
            val currentConfig:MutableMap<String,Set<String>> = signalConfig[maskSignal]?.toMutableMap() ?: mutableMapOf()
            val currentBacktraceList:MutableSet<String> = currentConfig[elfName]?.toMutableSet() ?: mutableSetOf()
            if (backtrace.isNotEmpty()){
                currentBacktraceList.add(backtrace)
            }
            currentConfig[elfName] = currentBacktraceList
            signalConfig[maskSignal] = currentConfig
            return this
        }

        /**
         * if only set elfName, but no any backtrace, will cover all of elfName's signal.
         */
        fun addSignalConfig(maskSignal: Int, elfName: String): Builder {
            val currentConfig:MutableMap<String,Set<String>> = signalConfig[maskSignal]?.toMutableMap() ?: mutableMapOf()
            val currentBacktraceList:MutableSet<String> = currentConfig[elfName]?.toMutableSet() ?: mutableSetOf()
            currentConfig[elfName] = currentBacktraceList
            signalConfig[maskSignal] = currentConfig
            return this
        }

        fun build(): AirbagConfig {
            val config = AirbagConfig()
            config.signalConfig = this.signalConfig
            return config
        }

        companion object {
            fun builder(): Builder = Builder()
        }
    }
}