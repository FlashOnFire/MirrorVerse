import 'dart:io';

import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';

enum MirrorType {
  plane,
  sphere,
}

void deleteFile(File file) {
  file.deleteSync();
}

void runGeneration(File file) async {
  //check if theres mirror_verse in the flutter assets
  try {
    final binary = await rootBundle.load("assets/mirror_verse_json");
    //write the file to tmp to be able to run it
    final dir = await getTemporaryDirectory();
    File("${dir.path}/mirror_verse_json")
        .writeAsBytesSync(binary.buffer.asUint8List());
    //make the file executable
    Process.runSync('chmod', ['+x', "${dir.path}/mirror_verse_json"]);
    //run the file
    Process.run("${dir.path}/mirror_verse_json", [file.path])
        .then((value) => File("${dir.path}/mirror_verse_json").deleteSync());
  } catch (e) {
    Process.run('cargo', ['run', '--release', file.path]);
  }
}

void generateMirrorSet(
    {required String name, required Map<MirrorType, int> mirrorCounts}) {
  print('Generating mirror set');
  print("Name: $name");
  print("Mirror counts: $mirrorCounts");
  final file = File('../assets/$name.json');
  // Process.run('cargo', ['run', '--release', file.path]);
}
