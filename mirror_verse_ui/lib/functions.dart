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

void runGeneration(File file, {List<String>? params}) async {
  // Check if there's mirror_verse in the flutter assets
  try {
    final binary = await rootBundle.load("assets/mirror_verse_json");
    // Write the file to tmp to be able to run it
    final dir = await getTemporaryDirectory();
    File("${dir.path}/mirror_verse_json")
        .writeAsBytesSync(binary.buffer.asUint8List());
    // Make the file executable
    if (Platform.isWindows) {
      Process.runSync('attrib', ['+x', "${dir.path}/mirror_verse_json"]);
    } else if (Platform.isLinux || Platform.isMacOS) {
      Process.runSync('chmod', ['+x', "${dir.path}/mirror_verse_json"]);
    } else {
      throw Exception("Unsupported platform");
    }
    // Run the file
    // if (Platform.isWindows) {
    //   Process.run("${dir.path}/mirror_verse_json", [
    //     if (params != null) ...params,
    //     file.path,
    //   ]).then((value) => File("${dir.path}/mirror_verse_json").deleteSync());
    // } else {
    Process.run("${dir.path}/mirror_verse_json", [
      if (params != null) ...params,
      file.path,
    ]).then((value) => File("${dir.path}/mirror_verse_json").deleteSync());
    // }
  } catch (e) {
    if (Platform.isWindows) {
      Process.run('cargo',
          ['run', '--release', '--', if (params != null) ...params, file.path]);
    } else {
      Process.run('cargo',
          ['run', '--release', '--', if (params != null) ...params, file.path]);
    }
  }
}

void generateMirrorSet(
    {required String name, required Map<MirrorType, int> mirrorCounts}) {
  final file = File('../assets/$name.json');
  List<String> params = ['-g'];
  mirrorCounts.forEach((key, value) {
    params.add('--${key.name}');
    params.add(value.toString());
  });
  runGeneration(file, params: params);
}
