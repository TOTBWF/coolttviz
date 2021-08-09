open GL
open Glu
open Glut

module J = Ezjsonm

open Basis
open Core
open Cubical
open CodeUnit
open Bwd


module S = Syntax

open Coolttviz.Hypercube

module IntMap = Map.Make (Int)

let init() =
  glClearColor 0.0 0.0 0.0 0.0;
  glShadeModel GL_FLAT

let label lbl x y z =
  glRasterPos3 x y z;
  lbl |> String.iter @@ fun c -> glutBitmapCharacter ~font:GLUT_BITMAP_9_BY_15 ~c

let cube (n : int) (r : float) =
  glBegin GL_QUADS;
  let points = hypercube n r in
  let _ = points |> List.iter @@ fun v ->
    let (x,y,z) = project v in
    glVertex3 x y z
  in
  glEnd()

let theta = ref 0.


(* Boundary Labeling *)

(* FIXME: Add something to Syntax.ml that lets us print boundary constraints nicer *)
(* FIXME: The dim var tracking code is attrocious *)
let dim_tm : S.t -> float =
  function
  | S.Dim0 -> -.1.0
  | S.Dim1 -> 1.0
  | _ -> failwith "dim_tm: bad dim"
let rec dim_from_cof (dims : (int option) bwd) (cof : S.t) : (int * float) list list =
  match cof with
  | S.Cof (Cof.Eq (S.Var v, r)) ->
    if (Option.is_none @@ Bwd.nth dims v) then
      Format.eprintf "[DEBUG] Could not find dimension for variable %i in %a@." v (pp_bwd (Format.pp_print_option Format.pp_print_int)) dims;
    let axis = Option.get @@ Bwd.nth dims v in
    let d = dim_tm r in
    [[(axis, d)]]
  | S.Cof (Cof.Join cofs) -> List.concat_map (dim_from_cof dims) cofs
  | S.Cof (Cof.Meet cofs) -> [List.concat @@ List.concat_map (dim_from_cof dims) cofs]
  | _ -> failwith "dim_from_cof: bad cof"

let boundary_point (n : int) (r : float) (bdry : (int * float) list) : float * float * float =
  let v = Array.make n 0.0 in
  let () = bdry |> List.iter @@ fun (dim, pos) ->
    v.(dim) <- r *. pos
  in
  project v

let ppenv_bind env ident =
  Pp.Env.bind env @@ Ident.to_string_opt ident

let render_boundary_constraints (dims : (int option) bwd) (env : Pp.env) (tm : S.t) : unit =
  let num_dims = Bwd.length @@ Bwd.filter Option.is_some dims in
  let rec go env dims (bdry, cons) =
    match cons with
    | S.CofSplit branches ->
      let (_x, envx) = ppenv_bind env `Anon in
      List.iter (go envx (Snoc (dims, None)))  branches
    | _ ->
      (* FIXME: This snoc is bad code!! *)
      let (_x, envx) = ppenv_bind env `Anon in
      dim_from_cof dims bdry |> List.iter @@ fun bdry_dims ->
      let (x,y,z) = boundary_point num_dims (1.0) bdry_dims in
      let lbl = Format.asprintf "%a" (S.pp envx) cons in
      label lbl x y z
  in
  match tm with
  | S.CofSplit branches ->
    let (_x, envx) = ppenv_bind env `Anon in
    List.iter (go envx (Snoc (dims, None))) branches
  | _ -> ()

(* What we want to do here is to say "var 3 is the first dimension" *)

let render_boundary (ctx : (Ident.t * S.tp) list) (goal : S.tp) : unit =
  let rec go dim_count dims env =
    function
    | [] ->
      begin
        match goal with
        | S.Sub (_, _, bdry) ->
          render_boundary_constraints dims env bdry;
          cube dim_count 1.0
        | _ -> ()
      end
    | (var, var_tp) :: ctx ->
      (* FIXME: Add the dims to the dim map *)
      let (_x, envx) = ppenv_bind env var in
      match var_tp with
      | S.TpDim -> go (dim_count + 1) (Snoc (dims, Some dim_count)) envx ctx
      | _ -> go dim_count (Snoc (dims, None)) envx ctx
  in go 0 Emp Pp.Env.emp ctx


let rotate_x = ref 0.0
let rotate_y = ref 0.0
let rotate_z = ref 0.0
let zoom = ref 1.0

let display ctx goal () =
  glClear ~mask:[GL_COLOR_BUFFER_BIT];
  glColor3 ~r:0.0 ~g:1.0 ~b:0.0;
  glLoadIdentity ();
  gluLookAt ~eyeX:0.0 ~eyeY:0.0 ~eyeZ:4.0 ~centerX:0.0 ~centerY:0.0 ~centerZ:0.0 ~upX:0.0 ~upY:1.0 ~upZ:0.0;
  glScale !zoom !zoom !zoom;

  glPolygonMode ~face:GL_FRONT_AND_BACK ~mode:GL_LINE;
  glPushMatrix();

  glRotate !rotate_z 0. 0. 1.;
  glRotate !rotate_y 1. 0. 0.;
  glRotate !rotate_x 0. 1. 0.;

  render_boundary ctx goal;
  glPopMatrix();
  glFlush ()

let reshape ~width:w ~height:h =
  glViewport ~x:0 ~y:0 ~width:w ~height:h; 
  glMatrixMode ~mode:GL_PROJECTION;
  glLoadIdentity ();
  glFrustum ~left:(-1.0) ~right:(1.0) ~bottom:(-1.0) ~top:(1.0) ~near:1.5 ~far:20.0;
  glMatrixMode ~mode:GL_MODELVIEW;
  glutPostRedisplay ()


(* Controls *)

let keyboard ~key ~x:_ ~y:_ =
  match key with
  | '\027' ->  (* exit *)
    exit(0);
  | '+' -> zoom := !zoom *. 1.1;
  | '-' -> zoom := !zoom *. 0.9;
  | _ ->
    Format.eprintf "[DEBUG] %c\n" key;
    ()

let left_click = ref GLUT_UP
let right_click = ref GLUT_UP
let xold = ref 0
let yold = ref 0

let mouse ~button ~state ~x ~y =
  xold := x;
  yold := y;
  match button with
  | GLUT_LEFT_BUTTON -> left_click := state;
  | GLUT_RIGHT_BUTTON -> right_click := state;
  | _ -> ()

let motion ~x ~y =
  if GLUT_DOWN = !left_click then
    begin
      rotate_y := !rotate_y +. float(y - !yold) /. 5.0;
      rotate_x := !rotate_x +. float(x - !xold) /. 5.0;
      if !rotate_y > 90. then
        rotate_y := 90.;
      if !rotate_y < -90. then
        rotate_y := -90.;
      glutPostRedisplay();
    end;
  if GLUT_DOWN = !right_click then
    begin
      rotate_z := !rotate_z +. float(x - !xold) /. 5.0;
      glutPostRedisplay();
    end;
  xold := x;
  yold := y;
;;

let idle () =
  rotate_x := !rotate_x +. 0.5;
  rotate_y := !rotate_x +. 0.5;
  theta := !theta +. 0.01;
  glutPostRedisplay()

let render_goal (ctx : (Ident.t * S.tp) list) (goal : S.tp) =
  let _ = glutInit ~argv:Sys.argv in
  glutInitDisplayMode ~mode:[GLUT_SINGLE; GLUT_RGB];
  glutInitWindowSize ~width:1000 ~height:1000;
  glutInitWindowPosition ~x:100 ~y:100;
  let _ = glutCreateWindow ~title:Sys.argv.(0) in
  init ();
  glutDisplayFunc ~display:(display ctx goal);
  glutReshapeFunc ~reshape;
  glutKeyboardFunc ~keyboard;
  glutMouseFunc ~mouse;
  glutMotionFunc ~motion;
  glutIdleFunc ~idle;
  glutMainLoop()

(* Goal Server *)

let server_init () =
  let conn_socket = Unix.socket Unix.PF_INET Unix.SOCK_STREAM 0 in
  let _ = Unix.bind conn_socket (Unix.ADDR_INET (Unix.inet_addr_of_string "127.0.0.1", 8080)) in
  let _ = Unix.listen conn_socket 1 in
  Format.eprintf "[DEBUG] Listening...@.";
  (* FIXME: We shouldn't do this *)
  let (socket, socket_addr) = Unix.accept conn_socket in
  Format.eprintf "[DEBUG] Accepted@.";
  let _ = Unix.close conn_socket in
  socket

let rec server_loop socket =
  let buf_size = 16384 in
  let buf = Bytes.create buf_size in
  let n_bytes = Unix.recv socket buf 0 buf_size [] in
  Format.eprintf "[DEBUG] Recieved (%i): %s@." n_bytes (Bytes.to_string buf);
  if n_bytes = 0 then
    Format.eprintf "[DEBUG] Goodbye!@."
  else
    let json = J.from_string (Bytes.to_string buf) in
    let (ctx, goal) = Serialize.deserialize_goal json in
    Format.eprintf "[DEBUG] %a@."  (Syntax.pp_sequent ~lbl:None ctx) goal;
    render_goal ctx goal;
    server_loop socket

(* let () =
 *   let socket = server_init () in
 *   server_loop socket *)
let display_lbl font () =
  glClear ~mask:[GL_COLOR_BUFFER_BIT];
  glColor3 ~r:0.0 ~g:1.0 ~b:0.0;
  glLoadIdentity ();
  gluLookAt ~eyeX:0.0 ~eyeY:0.0 ~eyeZ:4.0 ~centerX:0.0 ~centerY:0.0 ~centerZ:0.0 ~upX:0.0 ~upY:1.0 ~upZ:0.0;
  glScale !zoom !zoom !zoom;

  glPushMatrix();

  glRotate !rotate_z 0. 0. 1.;
  glRotate !rotate_y 1. 0. 0.;
  glRotate !rotate_x 0. 1. 0.;

  glRasterPos3 0. 0. 0.;
  Fonts.label font "a" 0. 0. 0.;
  (* label "a" 0. 0. 0.; *)

  glPopMatrix();
  glFlush ()

let () =
  let _ = glutInit ~argv:Sys.argv in
  glutInitDisplayMode ~mode:[GLUT_SINGLE; GLUT_RGB];
  glutInitWindowSize ~width:1000 ~height:1000;
  glutInitWindowPosition ~x:100 ~y:100;
  let _ = glutCreateWindow ~title:Sys.argv.(0) in
  init ();
  let font = Fonts.init "/Users/reedmullanix/Library/Fonts/iosevka-fixed-thin.ttf" in
  glutDisplayFunc ~display:(display_lbl font);
  glutReshapeFunc ~reshape;
  glutKeyboardFunc ~keyboard;
  glutMouseFunc ~mouse;
  glutMotionFunc ~motion;
  (* glutIdleFunc ~idle; *)
  glutMainLoop()
