Processes the output of Android app "SMS Backup & Restore", extracting images (and videos and vcards).

The app, to be specific, is https://play.google.com/store/apps/details?id=com.riteshsahu.SMSBackupRestore

It produces a giant XML file containing all your SMS messages, include MMS, and
appears to me to include RCS (in as much as messages that appear to be RCS
connected to me do, in fact, show up).

The XML includes the complete content of all MMS, including all images (base64 encoded).

I really, *really* like my image files to include a timestamp that actually
relates to, if nothing else, when *I* got the file, and manually saving files
from Messages on Android doesn't have any kind of timestamp info at all.

So rather than manually renaming some files, I wrote a thing to process a giant
XML blob to get the filename format I wanted.

That's normal, right?  Normal people do that?  Yeah, totally.

Anyway.

This program reads through that XML file and dumps all images it finds into the
output/ directory in the current directory.

It takes a single required argument, which is the file name for the XML input file.

It takes a second, optional, argument, which is the number of days to cover; so
"smsbar-images 30" will only get images from the most recent 30 days.
