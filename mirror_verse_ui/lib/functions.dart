import 'dart:io';

enum MirrorType {
  plane,
  sphere,
}

void deleteFile(File file) {
  file.deleteSync();
}

void runGeneration(File file) {
  Process.run('cargo', ['run', '--release', file.path]);
}

void generateMirrorSet(
    {required String name, required Map<MirrorType, int> mirrorCounts}) {
  print('Generating mirror set');
  print("Name: $name");
  print("Mirror counts: $mirrorCounts");
  final file = File('../assets/$name.json');
  // Process.runSync('cargo', ['run', '--release', file.path]);
}
