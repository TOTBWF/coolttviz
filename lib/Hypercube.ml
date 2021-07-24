
(** Generate all combinations of size k out of the set {0..n-1} *)
let combinations n k =
  let rec go i k =
    if k == 0 then [[]]
    else if i == n then []
    else (List.map (fun t -> i :: t) @@ go (i + 1) (k - 1)) @ (go (i + 1) k)
  in go 0 k

(** Iterate over the vertices of a hypercube, keeping a certain subset of the vertices fixed. *)
let iter_vertices (n : int) (r : float) (fixed : int list) (f : float list -> 'a) : 'a list =
  let rec go acc n fixed =
    match (n, fixed) with
    | (0, _) -> [f acc]
    | (n, (fx :: fixed)) when (n - 1) = fx -> go (0.0 :: acc) (n - 1) fixed
    | (n, fixed) -> (go (r :: acc) (n - 1) fixed) @ (go (-. r :: acc) (n - 1) fixed)
  in go [] n (List.sort (Fun.flip Int.compare) fixed)

(* FIXME: Using arrays here feels sketchy? *)

(** Generate all faces of a hypercube. *)
let hypercube (n : int) (size : float) : float array list =
  if (n == 1) then
    (* We still want to draw lines here, so let's special case this
       as a sort of degenerate 2-face
     *)
    [ [| -. size |]; [| -. size |]; [| size |]; [| size |] ]
  else
    let free_dims = combinations n 2 in
    free_dims |> List.concat_map @@ fun [@warning "-8"] [d0; d1] ->
    List.concat @@ iter_vertices n size [d0;d1] @@ fun point ->
      let point_buf = Array.of_list point in

      let bottom_left = Array.copy point_buf in
      Array.set bottom_left d0 (-. size);
      Array.set bottom_left d1 (-. size);
      let bottom_right = Array.copy point_buf in
      Array.set bottom_right d0 size;
      Array.set bottom_right d1 (-. size);
      let top_right = Array.copy point_buf in
      Array.set top_right d0 size;
      Array.set top_right d1 size;
      let top_left = Array.copy point_buf in
      Array.set top_left d0 (-. size);
      Array.set top_left d1 size;

      [ bottom_left;
        bottom_right;
        top_right;
        top_left;
      ]

(** Project down a single dimension *)
let project1 (v : float array) : float array =
  let view_angle = Float.pi /. 4.0 in
  let t = Float.tan(view_angle /. 2.0) in
  let proj =  Array.get v (Array.length v - 1) +. 3.0 in
  let r = Array.make (Array.length v - 1) 0.0 in
  for i = 0 to Array.length v - 2 do
    Array.set r i ((t *. Array.get v i) /. proj)
  done;
  r

let get_coord (v : float array) (i : int) : float =
  if Array.length v <= i then
    0.0
  else v.(i)


(** Project down into 3 dimensions *)
let project (v : float array) : float * float * float =
  let r = ref v in
  for _ = 0 to Array.length v - 4 do
    r := project1 !r
  done;
  (get_coord !r 0, get_coord !r 1, get_coord !r 2)

type 'a mat = 'a array array

let idmat (n : int) : float mat =
  let mat = Array.init n (fun _ -> Array.make n 0.0) in
  for i = 0 to n - 1 do
    mat.(i).(i) <- 1.0
  done;
  mat

let matmul (m : float mat) (v : float array) : float array =
  let r = Array.make (Array.length m) 0.0 in
  for i = 0 to Array.length m - 1 do
    let sum = ref 0.0 in
    for j = 0 to Array.length v - 1 do
      sum := !sum +. (m.(i).(j) *. v.(j));
    done;
    r.(i) <- !sum
  done;
  r

let rotmat (n : int) (axis0 : int) (axis1 : int) (theta : float) : float mat =
  let rot = idmat n in
  rot.(axis0).(axis0) <- Float.cos theta;
  rot.(axis0).(axis1) <- Float.sin theta;
  rot.(axis1).(axis0) <- -.Float.sin theta;
  rot.(axis1).(axis1) <- Float.cos theta;
  rot

let rotate (axis0 : int) (axis1 : int) (theta : float) (v : float array) : float array =
  matmul (rotmat (Array.length v) axis0 axis1 theta) v
