# Algoritmo de Eleccion del lider

## Algoritmo
1. Nodo que decide comenzar con la eleccion del lider

    - Busca en su lista de vecinos todos aquellos que tengan id mas alto (numero de puerto mas grande).
    - A todos ellos envia el mensaje de ELECTION
    - Se queda escuchando en el socket por respuestas sobre este ultimo mensaje
        - Si recibe OK, ya sabe que no puede ser COORDINATOR. Se queda escuchando el socket hasta el mensaje de COORDINATOR del punto (3)
        - Si no recibe nada (TIMEOUT), se autoproclama coordinador, seteando el puerto del lider por el propio y enviando el mensaje COORDINATOR a TODOS los otros nodos

2. Nodo que recibe ELECTION por el socket
    - Si `puerto_emisor` > `puerto_receptor`: No responde nada.
    - Si `puerto_emisor` < `puerto_receptor`: Responde OK al puerto_emisor. Inicia el proceso de ELECTION del punto (1)

3. Nodo que recibe COORDINATOR por el socket
    - Toma la direccion del emisor del mensaje y la setea como la del COORDINATOR.

## Momentos de ejecucion del algoritmo
1. Al inicio de la creacion de `BlockchainNode` (`BlockchainNode::new`)
   - Fuerza la deteccion del lider y la configuracion final del sistema previo a iniciar la carga de notas
2. Cuando un nodo detecta que el lider esta caido
   - Al enviarle un mensaje, no recibe una respuesta (esperando un tiempo TIMEOUT adecuado) 

## Para propositos de testing
- Se crea el comando `begin_election` que fuerza el proceso de eleccion de lider.