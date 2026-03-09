module dirent_d;

version (Win32_release) {

private import std.string;
private import std.c.stdlib;
private import std.c.windows.windows;

private struct DIR {
  HANDLE h;
  char* prev;
}

extern (C) DIR* opendir(char* name) {
  WIN32_FIND_DATA fd;
  char[] pattern = std.string.toString(name) ~ "/*";
  HANDLE h = FindFirstFileA(std.string.toStringz(pattern), &fd);
  DIR* d = cast(DIR*) malloc(DIR.sizeof);
  d.h = h;
  d.prev = null;
  return d;
}

extern (C) char* readdir_filename(DIR* d) {
  WIN32_FIND_DATA fd;
  BOOL ret = FindNextFileA(d.h, &fd);
  if (ret) {
    if (d.prev !is null) free(d.prev);
    char[] name = std.string.toString(fd.cFileName.ptr);
    d.prev = cast(char*) malloc(name.length + 1);
    d.prev[0 .. name.length] = name;
    d.prev[name.length] = '\0';
    return d.prev;
  } else {
    return null;
  }
}

extern (C) int closedir(DIR* d) {
  FindClose(d.h);
  if (d.prev !is null) free(d.prev);
  free(d);
  return 0;
}

} // version (Win32_release)
