mergeInto(LibraryManager.library, {
  start_javascript_play_sound: function(sound_id) {
	return play_sound(sound_id);
  },
  start_game: function() {
  	return start_game();
  },
  end_game: function() {
  	return end_game();
  },
});
