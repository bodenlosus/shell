{
  stdenv,
  gdk-pixbuf,
  webp-pixbuf-loader,
  gtk4,
}:
stdenv.mkDerivation {
  name = "custom-pixbuf-modfile";
  version = "1.0.0";
  nativeBuildInputs = [ gdk-pixbuf webp-pixbuf-loader];
  dontUnpack = true;
  buildPhase = ''
    gdk-pixbuf-query-loaders > loaders.cache
    
    gdk-pixbuf-query-loaders ${webp-pixbuf-loader}/lib/gdk-pixbuf-2.0/2.10.0/loaders/libpixbufloader-webp.so > loaders.cache
    mkdir -p $out
    mv ./loaders.cache $out/
  '';
}
