module.exports = function (grunt) {
  grunt.initConfig({
    uglify: {
      options: {
        mangle: true,
        compress: true,
      },
      build: {
        src: "static/*.js",
        dest: "dist/static",
        expand: true,
        flatten: true,
        ext: ".js",
      },
    },
    htmlmin: {
      dist: {
        options: {
          removeComments: true,
          collapseWhitespace: true,
        },
        files: [
          {
            expand: true,
            cwd: "./",
            src: ["index.html"],
            dest: "dist/",
          },
        ],
      },
    },
    copy: {
      main: {
        files: [
          {
            expand: true,
            cwd: "static/",
            src: ["**/*.png"],
            dest: "dist/static",
          },
        ],
      },
    },
  });

  grunt.loadNpmTasks("grunt-contrib-uglify");
  grunt.loadNpmTasks("grunt-contrib-htmlmin");
  grunt.loadNpmTasks("grunt-contrib-copy");

  grunt.registerTask("default", ["uglify", "htmlmin", "copy"]);
};
