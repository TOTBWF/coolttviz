open GL
open Glu
open Glut

open FreeType

type t =
  { textures : texture_id array;
    list_base : int;
    list_size : int;
  }


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
    for i = 0 to ((Bigarray.Array1.size_in_bytes bytes / 2) - 1) do
      output_byte chan @@ Bigarray.Array1.get bytes (i * 2)
    done;
    close_out chan
  with e ->
    close_out_noerr chan;
    raise e

let make_display_list (face : Face.t) (c : char) (list_base : int) (textures : texture_id array) =
  let ccode = Char.code c in
  let tex_id = textures.(ccode) in

  let _ = Face.load_glyph face (Face.get_char_index face (Int64.of_int @@ Char.code c)) [Face.Render] in
  let bitmap = Glyph.to_bitmap (Face.glyph face) RenderMode.Normal in
  let bytes = Bitmap.bytes bitmap in

  let width = pad_power (Bitmap.width bitmap) in
  let height = pad_power (Bitmap.height bitmap) in
  (* We make this twice as big to account for both luminosity and alpha data. *)
  let data =
    Bigarray.Array1.create Bigarray.int8_unsigned Bigarray.c_layout (2 * width * height)
  in
  for y = 0 to height - 1 do
    for x = 0 to width - 1 do
      (* Properly handle padding *)
      let px =
        if (x >= Bitmap.width bitmap || y >= Bitmap.height bitmap)
        then 0
        else
          Char.code @@ Bytes.get bytes (x + y*(Bitmap.width bitmap))
      in
      Bigarray.Array1.set data (2*(x + y*width)) px;
      Bigarray.Array1.set data (2*(x + y*width) + 1) px
    done
  done;
  Format.eprintf "Width: %n@.Height: %n@." (Bitmap.width bitmap) (Bitmap.height bitmap);

  glBindTexture GL_TEXTURE_2D tex_id;
  glTexParameter GL_TEXTURE_2D (GL_TEXTURE_MAG_FILTER GL_LINEAR);
  glTexParameter GL_TEXTURE_2D (GL_TEXTURE_MIN_FILTER GL_LINEAR);
  glTexImage2D GL_TEXTURE_2D 0 GL_RGBA width height GL_LUMINANCE_ALPHA GL_UNSIGNED_BYTE (Bigarray.genarray_of_array1 data);

  glNewList (list_base + ccode) GL_COMPILE;
  glBindTexture GL_TEXTURE_2D tex_id;
  (* glPushMatrix (); *)

  (* We should translate by the bitmap glyph info here! *)
  (* glTranslate __ __ 0.; *)

  let bwidth = Float.of_int @@ Bitmap.width bitmap in
  let bheight = Float.of_int @@ Bitmap.height bitmap in
  let x = bwidth /. (Float.of_int width) in
  let y = bheight /. (Float.of_int height) in

  glBegin GL_QUADS;
  glTexCoord2 0. 0.;
  glVertex2 0. bheight;

  glTexCoord2 0. y;
  glVertex2 0. 0.;

  glTexCoord2 x y;
  glVertex2 bwidth 0.;

  glTexCoord2 x 0.;
  glVertex2 bwidth bheight;

  glEnd ();

  (* glPopMatrix (); *)
  glEndList ()

let init path =
  let lib = Library.init () in
  let face = Face.create lib path 0 in
  Face.set_char_size face 0L (Int64.mul 12L 64L) 96 96;
  let list_size = 128 in
  let list_base = glGenLists list_size in
  let textures = glGenTextures list_size in
  for i = 0 to list_size - 1 do
    make_display_list face (Char.chr i) list_base textures
  done;
  Face.close face;
  Library.close lib;
  { textures; list_base; list_size }

let label t str x y z =
  glPushAttrib [GL_LIST_BIT; GL_CURRENT_BIT; GL_ENABLE_BIT; GL_TRANSFORM_BIT];
  glDisable GL_LIGHTING;
  glEnable GL_TEXTURE_2D;
  glDisable GL_DEPTH_TEST;
  glEnable GL_BLEND;
  glBlendFunc GL_SRC_ALPHA GL_ONE_MINUS_SRC_ALPHA;

  let lines = Array.of_seq @@ Seq.map Char.code @@ String.to_seq str in

  glListBase t.list_base;

  glCallLists lines;

  glPopAttrib ();

