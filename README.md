
## Enunciado

### Blockchain Rústica
Fecha de entrega: 27 de julio antes de las 19 hs.

### Objetivo
El objetivo del presente trabajo consiste en aplicar los conceptos estudiados de Concurrencia Distribuida para implementar una funcionalidad de blockchain simplificada.

### Introducción
La blockchain es una forma de almacenar información, que consiste en un registro único, consensuado y distribuido en varios nodos de una red.  A grandes rasgos su estructura es la de una cadena de bloques que están encadenados de forma sucesiva. Cada bloque contiene la información (a partir de un hash) del bloque que lo precede.

Se usa para almacenar información que no puede ser alterada, como es el caso de transacciones de criptomonedas. Se puede pensar como el libro contable donde se registra cada una de esas transacciones.

### Requerimientos Funcionales
Se debe implementar una versión reducida y simplificada de una blockchain que permita almacenar las calificaciones de los estudiantes de la materia. Esta información debe poder ser escrita y leída.

Se debe respetar que cada registro sea un bloque de la blockchain y, por lo tanto, tenga información referente al bloque o los bloques precedentes.

Esta implementación debe funcionar como un conjunto de programas en ejecución que se comunican entre sí utilizando sockets sobre el protocolo TCP/IP. Se debe utilizar Sockets de la biblioteca standar del lenguaje Rust.

Para el agregado de un bloque nuevo, un nodo debe poder realizar las operaciones de forma exclusiva. Para esto, se debe implementar alguno de los algoritmos estudiados en la materia:

* Algoritmo Centralizado
* Algoritmo Distribuido
* Algoritmo Token Ring

Una vez que se agrega un bloque nuevo, un nodo que cumple el rol de líder debe comunicar la operación a todos los demás. Para la elección del líder se debe implementar alguno de los algoritmos estudiados:

* Algoritmo de Bully
* Algoritmo Ring

Se debe poder simular la salida de servicio de los nodos de forma aleatoria o voluntaria. En particular, se debe poder observar que si sale de servicio el nodo que actúa como líder, se debe reiniciar el algoritmo de elección para reemplazarlo con uno nuevo.

### Requerimientos no funcionales
Los siguientes son los requerimientos no funcionales para la resolución de los ejercicios:

* El proyecto deberá ser desarrollado en lenguaje Rust, usando las herramientas de la biblioteca estándar.
* No se permite utilizar crates externos.
* El código fuente debe compilarse en la última versión stable del compilador y no se permite utilizar bloques unsafe.
* El código deberá funcionar en ambiente Unix / Linux.
* El programa deberá ejecutarse en la línea de comandos.
* La compilación no debe arrojar warnings del compilador, ni del linter clippy.
* Las funciones y los tipos de datos (struct) deben estar documentadas siguiendo el estándar de cargo doc.
* El código debe formatearse utilizando cargo fmt.
* Cada tipo de dato implementado debe ser colocado en una unidad de compilación (archivo fuente) independiente.

### Tareas a Realizar
A continuación se listan las tareas a realizar para completar el desarrollo del proyecto:

* Dividir el proyecto en procesos y threads. El objetivo es lograr procesos que cumplan un objetivo específico y que estos se conformen por un conjunto de hilos de ejecución que sean lo más sencillos posible.
* Una vez obtenida la división en threads, establecer un esquema de comunicación entre ellos teniendo en cuenta los requerimientos de la aplicación. ¿Qué threads se comunican entre sı́? ¿Qué datos necesitan compartir para poder trabajar?
* Realizar la codificación de la aplicación. El código fuente debe estar documentado.
* Implementar tests unitarios de las funciones que considere relevantes.

### Entrega
La resolución del presente proyecto es en grupos de tres integrantes.

La entrega del proyecto comprende lo siguiente:

* Informe, se deberá presentar en forma digital (PDF) enviado por correo electrónico a la dirección: pdeymon@fi.uba.ar
* El código fuente de la aplicación, que se entregará únicamente por e-mail. El código fuente debe estar estructurado en un proyecto de cargo, y se debe omitir el directorio target/ en la entrega. El informe a entregar debe contener los siguientes items:
    * Una explicación del diseño y de las decisiones tomadas para la implementación de la solución.
    * Detalle de resolución de la lista de tareas anterior.
    * Diagrama que refleje los threads, el flujo de comunicación entre ellos y los datos que intercambian.
    * Diagramas de entidades realizados (structs y demás).
    
## Uso

```
cargo build

# Start node 1
cargo run 6060 127.0.0.1:6061 127.0.0.1:6062

# Start node 2
cargo run 6061 127.0.0.1:6060 127.0.0.1:6062

# Start node 3
cargo run 6062 127.0.0.1:6060 127.0.0.1:6061
```