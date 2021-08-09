open GL
open Glu
open Glut

open FreeType

type t =
  { lib : Library.t;
    face : Face.t; }


(** OpenGL Textures /must/ be a power of 2 *)
let pad_power (n : int) : int =
  let rec go p =
    if p < n then go (Int.shift_left p 1) else p
  in go 1

let write_ppm path width height bytes =
  let chan = open_out_bin path in
  try
    (* Magic Bytes, we use P5 here to indicate that we are using the byte, grayscale version of the format. *)
    Printf.fprintf chan "P5\n";
    Printf.fprintf chan "%n %n\n" width height;
    (* Max color value, set to 255 as we will only be using grayscale. *)
    Printf.fprintf chan "255\n" ;
    output_bytes chan bytes;
    close_out chan
  with e ->
    close_out_noerr chan;
    raise e


let init path =
  let lib = Library.init () in
  let face = Face.create lib path 0 in
  Face.set_char_size face 0L (Int64.mul 12L 64L) 96 96;
  { lib; face }

(* FIXME: Horrible hack! *)
let shift_render_pos x y =
  glBitmap 0 0 0. 0. x y (Bigarray.Array1.create Bigarray.int8_unsigned Bigarray.c_layout 0)

(* We need to shift the advance along as we render characters *)
let render_char {face; _} : unit Uutf.String.folder =
  fun _ _ ->
  function
  | `Uchar uc ->
    let _ = Face.load_glyph face (Face.get_char_index face (Int64.of_int @@ Uchar.to_int uc)) [Face.Render] in
    let bitmap_glyph = Glyph.to_bitmap (Face.glyph face) RenderMode.Normal in
    let bitmap = BitmapGlyph.bitmap bitmap_glyph in
    let width = Bitmap.width bitmap in
    let height = Bitmap.height bitmap in
    let bytes = Bitmap.bytes bitmap in
    let alpha_bytes = Bytes.create (2 * Bitmap.width bitmap * Bitmap.height bitmap) in

    (* OpenGL will start drawing pixels from the bottom up, but our images 
    *)
    for y = 0 to height - 1 do
      for x = 0 to width - 1 do
        let b = Bytes.get bytes (x + y*width) in
        Bytes.set alpha_bytes (2*(x + (height - 1 - y)*width)) b;
        Bytes.set alpha_bytes (2*(x + (height - 1 - y)*width) + 1) b
      done
    done;

    let slot = Face.glyphslot face in
    let x_shift = Int.to_float @@ GlyphSlot.bitmap_left slot in
    let y_shift = Int.to_float @@ Bitmap.height bitmap - GlyphSlot.bitmap_top slot in

    shift_render_pos x_shift (-. y_shift);
    glDrawPixels_str (Bitmap.width bitmap) (Bitmap.height bitmap) GL_LUMINANCE_ALPHA GL_UNSIGNED_BYTE (Bytes.to_string alpha_bytes);
    let x_advance = Int64.to_float @@ Int64.shift_right (Vector.x @@ GlyphSlot.advance (Face.glyphslot face)) 6 in
    shift_render_pos (x_advance -. x_shift) (y_shift);

  | `Malformed _ -> failwith "render_char: Malformed unicode"

let label t str x y z =
  glPixelStorei GL_UNPACK_ALIGNMENT 2;
  glRasterPos3 x y z;
  Uutf.String.fold_utf_8 (render_char t) () str
