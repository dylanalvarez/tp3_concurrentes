# Algoritmo de Lock distribuido

## Algoritmo
- 1.1. Creo DistMutex pasandole el socket del coordinator. Va a tener condvars: 'acquiring' (solo usada si soy coordinator), 'taken', 'timeout_acquire', 'timeout_release'; y una queue: pending_locks (solo usada si soy coordinator)
- 1.2. Envio acquire() por el socket, arranco a contar timeout_acquire, y blockeo esta funcion con condvar: acquiring=true
   - 1.2.1. si se cumplio timeout_acquire sin haber sido reseteado (cuando se recibe el ok_acquire) --> disparar leader election y reintentar
- 1.3. Envio release() por el socket y no espero nada
- 1.4. Recibo acquire() por el socket
   - 1.4.1. Si soy el coordinador
      - 1.4.1.1. Si nadie tiene tomado lock (taken=false):
         - 1.4.1.1.1. pongo condvar taken=true
         - 1.4.1.1.2. arranco a contar timeout_release
            - 1.4.1.1.2.1. si se cumple el timeout_release (alguien que pidio el lock nunca lo liberó): poner condvar taken=false
         - 1.4.1.1.3. respondo ok_acquire() por el socket
      - 1.4.1.2. Si alguien tiene tomado el lock (taken=true): encolo la address que me vino por el socket en la queue pending_locks
   - 1.4.2. Si no soy el coordinador --> ignoro
- 1.5. Recibo ok_acquire() por el socket
   - 1.5.1. Si soy el coordinador o si no lo soy
      - 1.5.1.1. Pongo acquiring=false, reseteo timeout_acquire y dejo continuar al que llamo a la funcion acquire
- 1.6. Recibo release() por el socket
   - 1.6.1. Si soy el coordinador
      - 1.6.1.1. Si nadie tiene tomado el lock -> ignoro
      - 1.6.1.2. Si alguien tiene tomado el lock (taken=true)
         - 1.6.1.2.1. pongo condvar taken=false
         - 1.6.1.2.2. reseteo timeout_release
         - 1.6.1.2.3. repito hasta que queue pending_locks este vacía: desencolo de la queue pending_locks y vuelvo a 1.4
   - 1.6.2. Si no soy el coordinador --> ignoro