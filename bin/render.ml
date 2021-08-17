open Ctypes
open Foreign
open Signed
open Unsigned

open Basis
open Core
open Cubical
open CodeUnit
open Bwd

module S = Syntax

module IntMap = CCMap.Make(Int)

type label
let label : label structure typ = structure "label_"
let contents = field label "contents" string
let location = field label "location" (ptr float)
let () = seal label

let ppenv_bind env ident =
  Pp.Env.bind env @@ Ident.to_string_opt ident

let mk_label (n : int) (str : string) (axes : (int * float) list) : label structure =
  let lbl = make label in
  let pos = CArray.make float ~initial:0.0 n in
  setf lbl contents str;
  let _ = axes |> List.iter @@ fun (dim, p) ->
    CArray.set pos dim p
  in
  setf lbl location @@ CArray.start pos;
  lbl

let render_binding = foreign "render" (int @-> int @-> ptr label @-> returning int)

let dim_tm : S.t -> float =
  function
  | S.Dim0 -> -. 1.0
  | S.Dim1 -> 1.0
  | _ -> failwith "dim_tm: bad dim"

let rec dim_from_cof (dims : (int option) bwd) (cof : S.t) : (int * float) list list =
  match cof with
  | S.Cof (Cof.Eq (S.Var v, r)) ->
    let oaxis = Bwd.nth dims v in
    if (Option.is_none @@ oaxis) then
      Format.eprintf "[DEBUG] Could not find dimension for variable %i in %a@." v (pp_bwd (Format.pp_print_option Format.pp_print_int)) dims;
    let axis = Option.get oaxis in
    let d = dim_tm r in
    [[(axis, d)]]
  | S.Cof (Cof.Join cofs) -> List.concat_map (dim_from_cof dims) cofs
  | S.Cof (Cof.Meet cofs) -> [List.concat @@ List.concat_map (dim_from_cof dims) cofs]
  | _ -> failwith "dim_from_cof: bad cof"

let boundary_labels (num_dims : int) (dims : (int option) bwd) (env : Pp.env) (tm : S.t) : label structure list =
  let rec go env dims (bdry, cons) =
    match cons with
    | S.CofSplit branches ->
      let (_x, envx) = ppenv_bind env `Anon in
      List.concat_map (go envx (Snoc (dims, None))) branches
    | _ ->
      let (_x, envx) = ppenv_bind env `Anon in
      let lbl = Format.asprintf "%a" (S.pp envx) cons in
      List.map (mk_label num_dims lbl) @@ dim_from_cof (Snoc (dims, None)) bdry
  in
  match tm with
  | S.CofSplit branches ->
    let (_x, envx) = ppenv_bind env `Anon in
    List.concat_map (go envx dims) branches
  | _ -> []

let render_boundary (ctx : (Ident.t * S.tp) list) (goal : S.tp) : unit =
  let rec go dim_count dims env =
    function
    | [] ->
      begin
        match goal with
        | S.Sub (_, _, bdry) ->
          let num_dims = Bwd.length @@ Bwd.filter Option.is_some dims in
          let labels = boundary_labels num_dims dims env bdry in
          let label_ptr = CArray.start @@ CArray.of_list label labels in
          let _ = render_binding num_dims (List.length labels) label_ptr in
          ()
        | _ -> ()
      end
    | (var, var_tp) :: ctx ->
      let (_x, envx) = ppenv_bind env var in
      match var_tp with
      | S.TpDim -> go (dim_count + 1) (Snoc (dims, Some dim_count)) envx ctx
      | _ -> go dim_count (Snoc (dims, None)) envx ctx
  in go 0 Emp Pp.Env.emp ctx
