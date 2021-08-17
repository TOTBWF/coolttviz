module J = Ezjsonm

open Core
open CodeUnit

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
  let buf_size = 65536 in
  let buf = Bytes.create buf_size in
  let n_bytes = Unix.recv socket buf 0 buf_size [] in
  Format.eprintf "[DEBUG] Recieved (%i): %s@." n_bytes (Bytes.to_string buf);
  if n_bytes = 0 then
    Format.eprintf "[DEBUG] Goodbye!@."
  else
    let json = J.from_string (Bytes.to_string buf) in
    let (ctx, goal) = Serialize.deserialize_goal json in
    Format.eprintf "[DEBUG] %a@."  (Syntax.pp_sequent ~lbl:None ctx) goal;
    Render.render_boundary ctx goal;
    server_loop socket

let () =
  let socket = server_init () in
  server_loop socket
