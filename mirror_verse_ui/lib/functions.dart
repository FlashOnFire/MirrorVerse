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

Future<void> runExe(String exe, List<String>? params) async {
  // Check if there's mirror_verse in the flutter assets
  try {
    final binary = await rootBundle.load("assets/$exe");
    // Write the file to tmp to be able to run it
    final dir = await getTemporaryDirectory();
    File("${dir.path}/$exe").writeAsBytesSync(binary.buffer.asUint8List());
    // Make the file executable
    if (Platform.isWindows) {
      Process.runSync('attrib', ['+x', "${dir.path}/$exe"]);
    } else if (Platform.isLinux || Platform.isMacOS) {
      Process.runSync('chmod', ['+x', "${dir.path}/$exe"]);
    } else {
      throw Exception("Unsupported platform");
    }
    Process.run("${dir.path}/$exe", params ?? [])
        .then((value) => File("${dir.path}/$exe").deleteSync());
  } catch (e) {
    Process.run('cargo', [
      'run',
      '--release',
      '--bin',
      exe,
      '--',
      if (params != null) ...params,
    ]);
  }
}

Future<void> runGeneration(File file) async {
  await runExe("run_simulation_json_3d", [file.path]);
}

Future<void> generateMirrorSet(
    {required String name, required Map<MirrorType, int> mirrorCounts}) async {
  final file = File('../assets/$name.json');
  List<String> params = [];
  mirrorCounts.forEach((key, value) {
    params.add('--${key.name}');
    params.add(value.toString());
  });
  params = []; //TODO remove this line when we will have the correct parameters
  params.add(file.path);
  await runExe("generate_random_simulation_3d", params);
}
