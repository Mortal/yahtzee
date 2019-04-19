# setup.py based on https://github.com/getsentry/milksnake
from setuptools import setup


VERSION = '0.1.0'


def build_native(spec):
    build = spec.add_external_build(
        cmd=['cargo', 'build', '--release'],
        path='./rust',
    )

    spec.add_cffi_module(
        module_path='yahtzeevalue._native',
        dylib=lambda: build.find_dylib('yahtzeevalue', in_path='target/release'),
        header_filename=lambda: build.find_header('yahtzeevalue.h', in_path='include'),
    )


setup(
    name='yahtzeevalue',
    version=VERSION,
    packages=['yahtzeevalue'],
    author='Mathias Rav',
    license='GPL3+',
    author_email='m@git.strova.dk',
    description='A Python library for evaluating states in Yahtzee.',
    # long_description=readme,
    include_package_data=True,
    zip_safe=False,
    platforms='any',
    install_requires=['milksnake'],
    setup_requires=['milksnake'],
    milksnake_tasks=[
        build_native,
    ],
)
