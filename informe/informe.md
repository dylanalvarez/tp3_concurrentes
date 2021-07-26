# Trabajo Práctico 3 - Informe

|Integrante|Padrón|Mail|
|----------|------|----|
|Botalla, Tomás| 96356 | tbotalla@fi.uba.ar |
|Alvarez, Dylan| 98225 | dylanalvarez1995@gmail.com |
|Donato, Juan Pablo| 100839 | judonato@fi.uba.ar |

## Introducción

El objetivo de este informe será presentar y detallar las soluciones implementadas por el grupo 1 para la construcción del trabajo práctico 2, el cuál consistió en implementar una *blockchain rústica* de forma distribuida, considerando los distintos escenarios para lograr la implementación adecuada de dos algoritmos de **exclusion mútua distribuida** y de **elección de lider**.

En esta oportunidad, decidimos cumplir con la implementación de ambos algoritmos, utilizando:

- **Algoritmo de Bully**
- **Algoritmo Centralizado**

## Algoritmo de elección de lider - Algoritmo Bully

Para la parte de la elección de un nodo *lider* o *coordinador*, que será quien lleve las riendas de la sincronización para insertar elementos en la Blockchain de forma exitosa, decidimos implementarlo utilizando un algoritmo de tipo **Bully**, en donde prácticamente la elección del coordinador se basa en un criterio simple de que el nodo con mayor número de puerto en la red es quien será el coordinador. Dicho algoritmo se ejecuta ante la presencia de dos principales eventos:

- Cuando un nodo se levanta. Se ejecuta el algoritmo de elección para que los nodos descubran quien es el coordinador actual para que puedan comenzar a operar.
- Cuando un nodo detecta que el coordinador actual "esta caido". Si un nodo al enviarle mensajes realizando peticiones para insertar el siguiente dato en la cadena no recibe respuesta en un tiempo apropiado, entonces considerará al nodo coordinador como fuera de servicio, forzando una eleccion del lider nuevamente.

El proceso en cuestión implica:

1. El nodo que comienza la elección envia, solamente a quienes tengan número de puerto mas grande que el propio del nodo, un mensaje del tipo `ElectionMessage::Election` por la red, indicando el comienzo del proceso de elección de lider.
2. Cuando cada nodo reciba un mensaje de este tipo deberá responder al emisor con un mensaje `ElectionMessage::OkElection`, e iniciar el proceso de elección de lider, repitiendo el paso (1).
3. Si un determinado nodo NO recibe el mensaje `ElectionMessage::OkElection` de ningun otro nodo entonces *éste deberá proclamarse como coordinador*. En este caso, enviará por la red a TODOS los nodos restantes un mensaje `ElectionMessage::Coordinator`, autoproclamandose como lider o coordinador. Cuando los demás nodos lo reciban, actualizarán su referencia al nuevo nodo coordinador.

## Algoritmo de Exclusión Mutua - Algoritmo Centralizado