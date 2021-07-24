val hypercube : int -> float -> float array list
val project : float array -> float * float * float

type 'a mat

val idmat : int -> float mat
val matmul : float mat -> float array -> float array
val rotmat : int -> int -> int -> float -> float mat
val rotate : int -> int -> float -> float array -> float array
